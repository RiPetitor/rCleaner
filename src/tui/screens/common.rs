//! Общие компоненты экранов.

use crate::tui::action::SafetyLevel;
use crate::{NAME, VERSION};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

/// Отрисовывает заголовок приложения.
///
/// Макет:
/// ```text
/// ┌─────────────────────────────────────────────────────────┐
/// │ Safety: SAFE │       rCleaner        │ Bazzite 43 Atomic│
/// │              │         v0.9.0        │       | KDE      │
/// └─────────────────────────────────────────────────────────┘
/// ```
pub fn render_header(
    frame: &mut ratatui::Frame,
    area: Rect,
    system_label: &str,
    safety_level: SafetyLevel,
) {
    // Три колонки: Safety (слева), rCleaner (центр), Система (справа)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Левая часть — Safety
    let (safety_label, safety_color) = match safety_level {
        SafetyLevel::Safe => ("SAFE", Color::Green),
        SafetyLevel::Aggressive => ("AGGRESSIVE", Color::Yellow),
    };

    let left = Line::from(vec![
        Span::raw("  "),
        Span::styled("Safety: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            safety_label,
            Style::default()
                .fg(safety_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    // Центр — rCleaner + версия
    let center = Line::from(vec![
        Span::styled(
            NAME,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" v{VERSION}"), Style::default().fg(Color::DarkGray)),
    ]);

    // Правая часть — информация о системе
    let right = Line::from(vec![
        Span::styled(system_label, Style::default().fg(Color::White)),
        Span::raw("  "),
    ]);

    frame.render_widget(Paragraph::new(left).alignment(Alignment::Left), chunks[0]);
    frame.render_widget(
        Paragraph::new(center).alignment(Alignment::Center),
        chunks[1],
    );
    frame.render_widget(Paragraph::new(right).alignment(Alignment::Right), chunks[2]);
}
