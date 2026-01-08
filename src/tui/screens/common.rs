use crate::tui::action::SafetyLevel;
use crate::{NAME, VERSION};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

pub fn render_header(
    frame: &mut ratatui::Frame,
    area: Rect,
    system_label: &str,
    safety_level: SafetyLevel,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    let (safety_label, safety_color) = match safety_level {
        SafetyLevel::Safe => ("SAFE", Color::Green),
        SafetyLevel::Aggressive => ("AGGRESSIVE", Color::Yellow),
    };

    let left = Line::from(vec![
        Span::styled(
            NAME,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" v{VERSION}  ")),
        Span::styled(
            format!("Safety: {safety_label}"),
            Style::default().fg(safety_color),
        ),
    ]);

    let right = Line::from(Span::styled(
        system_label,
        Style::default().fg(Color::White),
    ));

    frame.render_widget(
        Paragraph::new(left)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true }),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(right)
            .alignment(Alignment::Right)
            .wrap(Wrap { trim: true }),
        chunks[1],
    );
}
