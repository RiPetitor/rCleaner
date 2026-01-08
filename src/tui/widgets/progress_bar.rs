use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge};

pub fn render_progress_bar(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    progress: f64,
    label: &str,
) {
    let gauge = Gauge::default()
        .block(Block::default().title(label).borders(Borders::ALL))
        .ratio(progress)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(gauge, area);
}
