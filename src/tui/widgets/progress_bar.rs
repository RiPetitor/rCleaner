use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge};

pub fn render_progress_bar(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    progress: f64,
    label: &str,
) {
    let percent = (progress * 100.0).clamp(0.0, 100.0);
    let label = format!("{label} ({percent:.0}%)");
    let color = if progress >= 1.0 {
        Color::Green
    } else {
        Color::Yellow
    };

    let gauge = Gauge::default()
        .block(Block::default().title(label).borders(Borders::ALL))
        .ratio(progress.clamp(0.0, 1.0))
        .label(format!("{percent:.0}%"))
        .style(Style::default().fg(color));

    frame.render_widget(gauge, area);
}
