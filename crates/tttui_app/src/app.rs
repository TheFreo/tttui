use std::collections::BTreeMap;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::{Frame, Terminal};
use tttui_core::{AppError, AppResult};

use crate::config::app_config::{AppConfig, PersonalBest};
use crate::features::preferences::application::ports::PreferencesRepository;
use crate::features::preferences::domain::keybinding::{KeyMap, KeySequenceMatcher};
use crate::features::preferences::domain::theme::{ResolvedTheme, ThemeDefinition};
use crate::features::preferences::infrastructure::file_preferences_repository::FilePreferencesRepository;
use crate::features::typing_test::application::ports::ContentRepository;
use crate::features::typing_test::application::use_cases::StartTypingTest;
use crate::features::typing_test::domain::result::TestResult;
use crate::features::typing_test::domain::session::TestSession;
use crate::features::typing_test::domain::test_mode::TestMode;
use crate::features::typing_test::infrastructure::file_content_repository::FileContentRepository;
use crate::features::typing_test::presentation::render::{render_result, render_test};

const SAMPLE_RATE: Duration = Duration::from_millis(250);

pub fn run() -> AppResult<()> {
    let preferences = FilePreferencesRepository::new()?;
    let config = preferences.load_config()?;
    let themes = preferences.load_themes()?;
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::InvalidConfig("could not determine config directory".into()))?
        .join("tttui");
    let content = FileContentRepository::new(config_dir)?;
    let mut app = App::new(config, themes, content)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = app.run(&mut terminal, &preferences);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

struct App<R>
where
    R: ContentRepository,
{
    config: AppConfig,
    themes: BTreeMap<String, ThemeDefinition>,
    theme: ResolvedTheme,
    keymap: KeyMap,
    matcher: KeySequenceMatcher,
    content: R,
    screen: Screen,
    home: HomeState,
}

impl<R> App<R>
where
    R: ContentRepository,
{
    fn new(
        config: AppConfig,
        themes: BTreeMap<String, ThemeDefinition>,
        content: R,
    ) -> AppResult<Self> {
        let theme = themes
            .get(&config.defaults.theme)
            .or_else(|| themes.get("default"))
            .or_else(|| themes.values().next())
            .ok_or_else(|| AppError::InvalidConfig("no themes are available".into()))?
            .resolve()?;
        let keymap = KeyMap::from_config(&config.keybindings)?;
        let languages = content.available_languages()?;
        let theme_names = themes.keys().cloned().collect::<Vec<_>>();
        let home = HomeState::new(&config, languages, theme_names);

        Ok(Self {
            config,
            themes,
            theme,
            keymap,
            matcher: KeySequenceMatcher::default(),
            content,
            screen: Screen::Home,
            home,
        })
    }

    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        preferences: &impl PreferencesRepository,
    ) -> AppResult<()> {
        loop {
            if let Screen::Test(session) = &mut self.screen {
                let now = Instant::now();
                session.tick(now);
                session.record_sample_if_due(now, SAMPLE_RATE);
                if session.is_complete() {
                    let result = session.result();
                    let key = format!("{}_{}", session.mode.key(), session.language);
                    let is_personal_best = self
                        .config
                        .personal_bests
                        .get(&key)
                        .map(|best| result.net_wpm > best.net_wpm)
                        .unwrap_or(true);
                    if is_personal_best {
                        self.config.personal_bests.insert(
                            key,
                            PersonalBest {
                                net_wpm: result.net_wpm,
                                raw_wpm: result.raw_wpm,
                                accuracy: result.accuracy,
                            },
                        );
                        preferences.save_config(&self.config)?;
                    }
                    self.screen = Screen::Result {
                        result,
                        is_personal_best,
                    };
                }
            }

            terminal.draw(|frame| self.render(frame))?;

            if !event::poll(Duration::from_millis(16))? {
                continue;
            }

            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if !self.handle_key(key, preferences)? {
                return Ok(());
            }
        }
    }

    fn handle_key(
        &mut self,
        key: KeyEvent,
        preferences: &impl PreferencesRepository,
    ) -> AppResult<bool> {
        match self.screen {
            Screen::Home => self.handle_home_key(key, preferences),
            Screen::Test(_) => self.handle_test_key(key),
            Screen::Result { .. } => self.handle_result_key(key, preferences),
        }
    }

    fn handle_home_key(
        &mut self,
        key: KeyEvent,
        preferences: &impl PreferencesRepository,
    ) -> AppResult<bool> {
        if let Some(action) = self.matcher.push_for_actions(
            &key,
            &self.keymap,
            &[
                "focus_next",
                "focus_previous",
                "cycle_next",
                "cycle_previous",
                "start",
                "quit",
            ],
        ) {
            match action.as_str() {
                "focus_next" => self.home.focus_next(),
                "focus_previous" => self.home.focus_previous(),
                "cycle_next" => self.home.cycle_next(),
                "cycle_previous" => self.home.cycle_previous(),
                "start" => self.start_test(preferences)?,
                "quit" => return Ok(false),
                _ => {}
            }
        }
        Ok(true)
    }

    fn handle_test_key(&mut self, key: KeyEvent) -> AppResult<bool> {
        if let Some(action) =
            self.matcher
                .push_for_actions(&key, &self.keymap, &["restart", "menu"])
        {
            match action.as_str() {
                "restart" => {
                    self.start_test_with_current_config()?;
                    return Ok(true);
                }
                "menu" => {
                    self.screen = Screen::Home;
                    return Ok(true);
                }
                _ => {}
            }
        }

        if let Screen::Test(session) = &mut self.screen {
            match key.code {
                KeyCode::Backspace => session.backspace(),
                KeyCode::Char(value) => {
                    session.start_if_needed(Instant::now());
                    session.push_char(value);
                }
                _ => {}
            }
        }

        Ok(true)
    }

    fn handle_result_key(
        &mut self,
        key: KeyEvent,
        preferences: &impl PreferencesRepository,
    ) -> AppResult<bool> {
        if let Some(action) =
            self.matcher
                .push_for_actions(&key, &self.keymap, &["start", "focus_next", "quit"])
        {
            match action.as_str() {
                "start" => self.start_test(preferences)?,
                "focus_next" => self.screen = Screen::Home,
                "quit" => return Ok(false),
                _ => {}
            }
        }
        Ok(true)
    }

    fn start_test(&mut self, preferences: &impl PreferencesRepository) -> AppResult<()> {
        self.config.defaults.mode = self.home.mode_name().into();
        self.config.defaults.duration = self.home.current_duration();
        self.config.defaults.word_count = self.home.current_word_count();
        self.config.defaults.language = self.home.current_language().into();
        self.config.defaults.theme = self.home.current_theme().into();
        preferences.save_config(&self.config)?;
        self.start_test_with_current_config()
    }

    fn start_test_with_current_config(&mut self) -> AppResult<()> {
        self.config.defaults.mode = self.home.mode_name().into();
        self.config.defaults.duration = self.home.current_duration();
        self.config.defaults.word_count = self.home.current_word_count();
        self.config.defaults.language = self.home.current_language().into();
        self.config.defaults.theme = self.home.current_theme().into();

        let theme = self
            .themes
            .get(self.home.current_theme())
            .ok_or_else(|| AppError::InvalidConfig("selected theme does not exist".into()))?
            .resolve()?;
        self.theme = theme;
        let mode = self.home.current_mode();
        let use_case = StartTypingTest::new(&self.content);
        self.screen = Screen::Test(use_case.execute(mode, self.home.current_language())?);
        self.matcher.clear();
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        match &self.screen {
            Screen::Home => render_home(frame, area, &self.home, &self.theme),
            Screen::Test(session) => render_test(frame, area, session, &self.theme),
            Screen::Result {
                result,
                is_personal_best,
            } => render_result(frame, area, result, *is_personal_best, &self.theme),
        }
    }
}

enum Screen {
    Home,
    Test(TestSession),
    Result {
        result: TestResult,
        is_personal_best: bool,
    },
}

#[derive(Debug)]
struct HomeState {
    focus: usize,
    mode_index: usize,
    duration_index: usize,
    word_count_index: usize,
    language_index: usize,
    theme_index: usize,
    durations: Vec<u16>,
    word_counts: Vec<u16>,
    languages: Vec<String>,
    themes: Vec<String>,
}

impl HomeState {
    fn new(config: &AppConfig, languages: Vec<String>, themes: Vec<String>) -> Self {
        let mode_index = match config.defaults.mode.as_str() {
            "words" => 1,
            "quote" => 2,
            _ => 0,
        };
        let duration_index = index_or_zero(&config.options.durations, config.defaults.duration);
        let word_count_index =
            index_or_zero(&config.options.word_counts, config.defaults.word_count);
        let language_index = index_or_zero(&languages, config.defaults.language.clone());
        let theme_index = index_or_zero(&themes, config.defaults.theme.clone());

        Self {
            focus: 0,
            mode_index,
            duration_index,
            word_count_index,
            language_index,
            theme_index,
            durations: config.options.durations.clone(),
            word_counts: config.options.word_counts.clone(),
            languages,
            themes,
        }
    }

    fn focus_next(&mut self) {
        self.focus = (self.focus + 1) % 4;
    }

    fn focus_previous(&mut self) {
        self.focus = self.focus.checked_sub(1).unwrap_or(3);
    }

    fn cycle_next(&mut self) {
        self.cycle(1);
    }

    fn cycle_previous(&mut self) {
        self.cycle(-1);
    }

    fn cycle(&mut self, delta: isize) {
        match self.focus {
            0 => self.mode_index = cycle_index(self.mode_index, 3, delta),
            1 => match self.mode_index {
                0 => {
                    self.duration_index =
                        cycle_index(self.duration_index, self.durations.len(), delta)
                }
                1 => {
                    self.word_count_index =
                        cycle_index(self.word_count_index, self.word_counts.len(), delta)
                }
                _ => {}
            },
            2 => {
                self.language_index = cycle_index(self.language_index, self.languages.len(), delta)
            }
            3 => self.theme_index = cycle_index(self.theme_index, self.themes.len(), delta),
            _ => {}
        }
    }

    fn mode_name(&self) -> &'static str {
        match self.mode_index {
            1 => "words",
            2 => "quote",
            _ => "time",
        }
    }

    fn current_mode(&self) -> TestMode {
        match self.mode_index {
            1 => TestMode::Words(self.current_word_count()),
            2 => TestMode::Quote,
            _ => TestMode::Time(self.current_duration()),
        }
    }

    fn current_duration(&self) -> u16 {
        self.durations[self.duration_index]
    }

    fn current_word_count(&self) -> u16 {
        self.word_counts[self.word_count_index]
    }

    fn current_language(&self) -> &str {
        &self.languages[self.language_index]
    }

    fn current_theme(&self) -> &str {
        &self.themes[self.theme_index]
    }

    fn length_label(&self) -> String {
        match self.mode_index {
            1 => self.current_word_count().to_string(),
            2 => "quote".into(),
            _ => self.current_duration().to_string(),
        }
    }
}

fn render_home(frame: &mut Frame, area: Rect, home: &HomeState, theme: &ResolvedTheme) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new("tttui").alignment(Alignment::Center).style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        centered_line(sections[0]),
    );

    let labels = [
        home.mode_name().to_string(),
        home.length_label(),
        home.current_language().to_string(),
        home.current_theme().to_string(),
    ];
    let mut spans = Vec::new();
    for (index, label) in labels.into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(
                theme.presentation.selector_separator.clone(),
                Style::default().fg(theme.muted),
            ));
        }
        let style = if index == home.focus {
            Style::default()
                .fg(theme.selection)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(theme.text)
        };
        spans.push(Span::styled(label, style));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        sections[1],
    );

    frame.render_widget(
        Paragraph::new("tab focus   left/right change   enter start")
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.muted)),
        sections[3],
    );
}

fn centered_line(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y + area.height / 2,
        width: area.width,
        height: 1,
    }
}

fn cycle_index(index: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }

    (index as isize + delta).rem_euclid(len as isize) as usize
}

fn index_or_zero<T: PartialEq>(values: &[T], expected: T) -> usize {
    values
        .iter()
        .position(|value| *value == expected)
        .unwrap_or(0)
}
