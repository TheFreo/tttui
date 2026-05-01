use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols::{border, Marker};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Wrap};
use ratatui::Frame;

use crate::features::preferences::domain::theme::ResolvedTheme;
use crate::features::typing_test::domain::result::TestResult;
use crate::features::typing_test::domain::session::TestSession;

pub fn render_test(frame: &mut Frame, area: Rect, session: &TestSession, theme: &ResolvedTheme) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .horizontal_margin(2)
        .split(area);

    let timer = match session.mode {
        crate::features::typing_test::domain::test_mode::TestMode::Time(duration) => {
            let remaining = duration as f64 - session.elapsed.as_secs_f64();
            format!("{:.1}s", remaining.max(0.0))
        }
        _ => format!("{:.1}s", session.elapsed.as_secs_f64()),
    };

    let header = Line::from(vec![
        Span::styled(
            session.mode.label(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ", Style::default()),
        Span::styled(&session.language, Style::default().fg(theme.muted)),
        Span::styled("   ", Style::default()),
        Span::styled(timer, Style::default().fg(theme.muted)),
        Span::styled("   ", Style::default()),
        Span::styled(
            format!("{:.0} wpm", session.current_net_wpm()),
            Style::default().fg(theme.muted),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(header).alignment(Alignment::Center),
        sections[0],
    );

    let available_width = sections[1].width.saturating_sub(4).max(20) as usize;
    let lines = wrap_target(session, available_width);
    let active_line = active_line_index(session, &lines);
    let visible_start = active_line.saturating_sub(1);
    let visible_end = (visible_start + 3).min(lines.len());
    let visible = &lines[visible_start..visible_end];

    let rendered_lines = visible
        .iter()
        .map(|(start, chars)| render_target_line(session, *start, chars, theme))
        .collect::<Vec<_>>();

    let paragraph = Paragraph::new(rendered_lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, centered_text_area(sections[1], 3));

    let command = Line::from(vec![
        Span::styled("tab enter", Style::default().fg(theme.accent)),
        Span::styled(" restart   ", Style::default().fg(theme.muted)),
        Span::styled("tab m", Style::default().fg(theme.accent)),
        Span::styled(" menu", Style::default().fg(theme.muted)),
    ]);
    frame.render_widget(
        Paragraph::new(command).alignment(Alignment::Center),
        sections[2],
    );
}

pub fn render_result(
    frame: &mut Frame,
    area: Rect,
    result: &TestResult,
    is_personal_best: bool,
    theme: &ResolvedTheme,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .horizontal_margin(2)
        .split(area);

    let title = if is_personal_best {
        format!("{:.2} wpm  personal best", result.net_wpm)
    } else {
        format!("{:.2} wpm", result.net_wpm)
    };
    frame.render_widget(
        Paragraph::new(title)
            .style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center),
        sections[0],
    );

    let stats = vec![
        Line::from(format!(
            "raw {:.2}   acc {:.2}%   consistency {:.2}%",
            result.raw_wpm, result.accuracy, result.consistency
        )),
        Line::from(format!(
            "time {:.2}s   chars {}",
            result.duration.as_secs_f64(),
            result.char_summary()
        )),
    ];
    frame.render_widget(
        Paragraph::new(stats)
            .style(Style::default().fg(theme.text))
            .alignment(Alignment::Center),
        sections[1],
    );

    let graph_area = sections[2];
    let data = smoothed_history(&result.history);
    let max_wpm = data
        .iter()
        .map(|(_, value)| *value)
        .fold(0.0_f64, f64::max)
        .max(10.0);
    let rounded_max_wpm = round_axis_max(max_wpm);
    let duration = result.duration.as_secs_f64().max(1.0);
    if data.len() < 2 {
        frame.render_widget(
            Paragraph::new("not enough samples for graph")
                .style(Style::default().fg(theme.muted))
                .alignment(Alignment::Center),
            centered_text_area(graph_area, 1),
        );
    } else {
        let y_ticks = wpm_ticks(rounded_max_wpm, graph_area.height);
        let x_ticks = time_ticks(duration, graph_area.width);
        let guide_lines = guide_line_data(&y_ticks, duration);
        let mut datasets = guide_datasets(&guide_lines, theme);
        datasets.push(
            Dataset::default()
                .style(Style::default().fg(theme.correct))
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .data(&data),
        );
        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .bounds([0.0, duration])
                    .labels(
                        x_ticks
                            .iter()
                            .map(|value| Line::from(format!("{value:.0}s")))
                            .collect::<Vec<_>>(),
                    )
                    .style(Style::default().fg(theme.muted)),
            )
            .y_axis(
                Axis::default()
                    .bounds([0.0, rounded_max_wpm])
                    .labels(
                        y_ticks
                            .iter()
                            .map(|value| Line::from(format!("{value:.0}")))
                            .collect::<Vec<_>>(),
                    )
                    .style(Style::default().fg(theme.muted)),
            )
            .block(optional_block(theme));
        frame.render_widget(chart, graph_area);
    }

    frame.render_widget(
        Paragraph::new("enter retry   tab menu   q quit")
            .style(Style::default().fg(theme.muted))
            .alignment(Alignment::Center),
        sections[3],
    );
}

fn centered_text_area(area: Rect, height: u16) -> Rect {
    let top = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x: area.x,
        y: top,
        width: area.width,
        height: area.height.min(height),
    }
}

fn wrap_target(session: &TestSession, width: usize) -> Vec<(usize, Vec<char>)> {
    let mut lines = Vec::new();
    let mut current = Vec::new();
    let mut start = 0;
    let mut word_start = 0;

    for (index, ch) in session.target.iter().copied().enumerate() {
        if ch == ' ' {
            word_start = current.len() + 1;
        }

        current.push(ch);

        if current.len() >= width {
            let split_at = if word_start > 0 {
                word_start
            } else {
                current.len()
            };
            let next = current.split_off(split_at);
            lines.push((start, current));
            start = index + 1 - next.len();
            current = next;
            word_start = 0;
        }
    }

    if !current.is_empty() {
        lines.push((start, current));
    }

    lines
}

fn active_line_index(session: &TestSession, lines: &[(usize, Vec<char>)]) -> usize {
    let caret = session.input.len();
    lines
        .iter()
        .position(|(start, chars)| caret >= *start && caret <= start + chars.len())
        .unwrap_or_else(|| lines.len().saturating_sub(1))
}

fn render_target_line(
    session: &TestSession,
    start: usize,
    chars: &[char],
    theme: &ResolvedTheme,
) -> Line<'static> {
    let spans = chars
        .iter()
        .enumerate()
        .map(|(offset, expected)| {
            let index = start + offset;
            let style = match session.input.get(index) {
                Some(typed) if typed == expected => Style::default().fg(theme.correct),
                Some(_) => Style::default().fg(theme.incorrect),
                None if index == session.input.len() => Style::default()
                    .fg(theme.caret)
                    .add_modifier(Modifier::UNDERLINED),
                None => Style::default().fg(theme.untyped),
            };
            Span::styled(expected.to_string(), style)
        })
        .collect::<Vec<_>>();

    Line::from(spans)
}

fn optional_block(theme: &ResolvedTheme) -> Block<'static> {
    if theme.presentation.show_borders {
        let block = Block::default().borders(Borders::ALL);
        match theme.presentation.border_style.as_str() {
            "rounded" => block.border_set(border::ROUNDED),
            "double" => block.border_set(border::DOUBLE),
            "thick" => block.border_set(border::THICK),
            _ => block,
        }
    } else {
        Block::default()
    }
}

fn smoothed_history(history: &[(std::time::Duration, f64)]) -> Vec<(f64, f64)> {
    if history.is_empty() {
        return Vec::new();
    }

    let window = (history.len() / 6).max(1);
    history
        .iter()
        .enumerate()
        .map(|(index, (time, _))| {
            let start = index.saturating_sub(window / 2);
            let end = (index + window / 2 + 1).min(history.len());
            let average = history[start..end]
                .iter()
                .map(|(_, value)| value)
                .sum::<f64>()
                / (end - start) as f64;
            (time.as_secs_f64(), average)
        })
        .collect()
}

fn round_axis_max(value: f64) -> f64 {
    (value / 10.0).ceil().max(1.0) * 10.0
}

fn wpm_ticks(max_wpm: f64, graph_height: u16) -> Vec<f64> {
    let max_labels = graph_height.saturating_sub(1).clamp(2, 7) as usize;
    let step = [10.0, 20.0, 25.0, 50.0, 100.0, 200.0]
        .into_iter()
        .find(|candidate| ((max_wpm / candidate).ceil() as usize + 1) <= max_labels)
        .unwrap_or(200.0);
    evenly_spaced_ticks(max_wpm, step)
}

fn time_ticks(duration: f64, graph_width: u16) -> Vec<f64> {
    let max_labels = (graph_width / 10).clamp(2, 7) as usize;
    let step = [1.0, 2.0, 5.0, 10.0, 15.0, 20.0, 30.0, 60.0, 120.0]
        .into_iter()
        .find(|candidate| ((duration / candidate).ceil() as usize + 1) <= max_labels)
        .unwrap_or(120.0);
    evenly_spaced_ticks(duration, step)
}

fn evenly_spaced_ticks(max_value: f64, step: f64) -> Vec<f64> {
    let rounded_max = (max_value / step).ceil() * step;
    let mut ticks = Vec::new();
    let mut value = 0.0;

    while value < rounded_max {
        ticks.push(value);
        value += step;
    }
    ticks.push(rounded_max);
    ticks
}

fn guide_line_data(ticks: &[f64], duration: f64) -> Vec<[(f64, f64); 2]> {
    ticks
        .iter()
        .copied()
        .filter(|value| *value > 0.0 && *value < *ticks.last().unwrap_or(&0.0))
        .map(|value| [(0.0, value), (duration, value)])
        .collect()
}

fn guide_datasets<'a>(
    guide_lines: &'a [[(f64, f64); 2]],
    theme: &ResolvedTheme,
) -> Vec<Dataset<'a>> {
    guide_lines
        .iter()
        .map(|line| {
            Dataset::default()
                .style(Style::default().fg(theme.muted).add_modifier(Modifier::DIM))
                .marker(Marker::Dot)
                .graph_type(GraphType::Line)
                .data(line)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn smooths_history_without_losing_time_axis() {
        let history = vec![
            (Duration::from_secs(1), 20.0),
            (Duration::from_secs(2), 40.0),
            (Duration::from_secs(3), 60.0),
        ];
        let smoothed = smoothed_history(&history);

        assert_eq!(smoothed.len(), 3);
        assert_eq!(smoothed[0].0, 1.0);
        assert_eq!(smoothed[2].0, 3.0);
    }

    #[test]
    fn rounds_graph_axis_to_tens() {
        assert_eq!(round_axis_max(1.0), 10.0);
        assert_eq!(round_axis_max(46.0), 50.0);
    }

    #[test]
    fn uses_readable_wpm_ticks_when_possible() {
        assert_eq!(
            wpm_ticks(120.0, 12),
            vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0, 120.0]
        );
        assert_eq!(wpm_ticks(180.0, 8), vec![0.0, 50.0, 100.0, 150.0, 200.0]);
    }

    #[test]
    fn uses_time_ticks_based_on_duration() {
        assert_eq!(
            time_ticks(30.0, 80),
            vec![0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0]
        );
        assert_eq!(
            time_ticks(120.0, 80),
            vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0, 120.0]
        );
    }
}
