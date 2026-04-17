use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use tttui_core::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeDefinition {
    #[serde(default)]
    pub colors: ThemeColors,
    #[serde(default)]
    pub presentation: ThemePresentation,
}

impl Default for ThemeDefinition {
    fn default() -> Self {
        Self {
            colors: ThemeColors::default(),
            presentation: ThemePresentation::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    #[serde(default = "default_text")]
    pub text: String,
    #[serde(default = "default_muted")]
    pub muted: String,
    #[serde(default = "default_correct")]
    pub correct: String,
    #[serde(default = "default_incorrect")]
    pub incorrect: String,
    #[serde(default = "default_untyped")]
    pub untyped: String,
    #[serde(default = "default_caret")]
    pub caret: String,
    #[serde(default = "default_accent")]
    pub accent: String,
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default = "default_selection")]
    pub selection: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            text: default_text(),
            muted: default_muted(),
            correct: default_correct(),
            incorrect: default_incorrect(),
            untyped: default_untyped(),
            caret: default_caret(),
            accent: default_accent(),
            background: default_background(),
            selection: default_selection(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePresentation {
    #[serde(default = "default_border_style")]
    pub border_style: String,
    #[serde(default)]
    pub show_borders: bool,
    #[serde(default = "default_selector_separator")]
    pub selector_separator: String,
    #[serde(default = "default_caret_symbol")]
    pub caret_symbol: String,
}

impl Default for ThemePresentation {
    fn default() -> Self {
        Self {
            border_style: default_border_style(),
            show_borders: false,
            selector_separator: default_selector_separator(),
            caret_symbol: default_caret_symbol(),
        }
    }
}

impl ThemeDefinition {
    pub fn color(&self, value: &str) -> AppResult<Color> {
        parse_color(value)
    }
}

pub fn parse_color(value: &str) -> AppResult<Color> {
    let lower = value.trim().to_ascii_lowercase();
    let parsed = match lower.as_str() {
        "reset" | "default" => Color::Reset,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "white" => Color::White,
        value if value.starts_with('#') && value.len() == 7 => {
            let red = u8::from_str_radix(&value[1..3], 16)
                .map_err(|_| AppError::InvalidConfig(format!("invalid color `{value}`")))?;
            let green = u8::from_str_radix(&value[3..5], 16)
                .map_err(|_| AppError::InvalidConfig(format!("invalid color `{value}`")))?;
            let blue = u8::from_str_radix(&value[5..7], 16)
                .map_err(|_| AppError::InvalidConfig(format!("invalid color `{value}`")))?;
            Color::Rgb(red, green, blue)
        }
        value => {
            let index = value
                .parse::<u8>()
                .map_err(|_| AppError::InvalidConfig(format!("invalid color `{value}`")))?;
            Color::Indexed(index)
        }
    };

    Ok(parsed)
}

fn default_text() -> String {
    "white".into()
}

fn default_muted() -> String {
    "darkgray".into()
}

fn default_correct() -> String {
    "green".into()
}

fn default_incorrect() -> String {
    "red".into()
}

fn default_untyped() -> String {
    "gray".into()
}

fn default_caret() -> String {
    "yellow".into()
}

fn default_accent() -> String {
    "cyan".into()
}

fn default_background() -> String {
    "default".into()
}

fn default_selection() -> String {
    "blue".into()
}

fn default_border_style() -> String {
    "plain".into()
}

fn default_selector_separator() -> String {
    " / ".into()
}

fn default_caret_symbol() -> String {
    "_".into()
}
