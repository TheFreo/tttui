use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::config::app_config::SessionHistoryEntry;
use crate::features::preferences::domain::theme::ResolvedTheme;

pub fn render_history(
    frame: &mut Frame,
    area: Rect,
    entries: &[SessionHistoryEntry],
    theme: &ResolvedTheme,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .horizontal_margin(2)
        .split(area);

    frame.render_widget(
        Paragraph::new("history")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        sections[0],
    );

    if entries.is_empty() {
        frame.render_widget(
            Paragraph::new("no completed tests yet")
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.muted)),
            centered_line(sections[1]),
        );
    } else {
        let max_rows = sections[1].height as usize;
        let now = unix_now();
        let lines = entries
            .iter()
            .take(max_rows)
            .map(|entry| history_line(entry, now, theme))
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(lines)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            sections[1],
        );
    }

    frame.render_widget(
        Paragraph::new("tab back   q quit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.muted)),
        sections[2],
    );
}

fn history_line(entry: &SessionHistoryEntry, now: u64, theme: &ResolvedTheme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            relative_age(entry.completed_at_unix, now),
            Style::default().fg(theme.muted),
        ),
        Span::raw("   "),
        Span::styled(entry.mode.clone(), Style::default().fg(theme.text)),
        Span::raw("   "),
        Span::styled(
            format!("{:.2} wpm", entry.net_wpm),
            Style::default().fg(theme.correct),
        ),
        Span::raw("   "),
        Span::styled(
            format!("{:.2}% acc", entry.accuracy),
            Style::default().fg(theme.text),
        ),
        Span::raw("   "),
        Span::styled(entry.language.clone(), Style::default().fg(theme.muted)),
    ])
}

fn centered_line(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y + area.height / 2,
        width: area.width,
        height: 1,
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn relative_age(completed_at: u64, now: u64) -> String {
    let age = now.saturating_sub(completed_at);
    match age {
        0..=59 => "now".into(),
        60..=3_599 => format!("{}m", age / 60),
        3_600..=86_399 => format!("{}h", age / 3_600),
        _ => format!("{}d", age / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_compact_relative_ages() {
        assert_eq!(relative_age(100, 100), "now");
        assert_eq!(relative_age(100, 220), "2m");
        assert_eq!(relative_age(100, 7_300), "2h");
        assert_eq!(relative_age(100, 172_900), "2d");
    }
}
