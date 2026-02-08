use crate::tui::screens::common::Theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Gauge};

pub fn render_progress_bar(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    progress: f64,
    label: &str,
) {
    let percent = (progress * 100.0).clamp(0.0, 100.0);
    let color = if progress >= 1.0 {
        Theme::SUCCESS
    } else {
        Theme::ACCENT
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" {label} "),
                    Style::default()
                        .fg(Theme::TEXT)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Theme::BORDER)),
        )
        .ratio(progress.clamp(0.0, 1.0))
        .label(Span::styled(
            format!("{percent:.0}%"),
            Style::default()
                .fg(Theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ))
        .gauge_style(Style::default().fg(color).bg(Theme::BG_HIGHLIGHT));

    frame.render_widget(gauge, area);
}
