use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render_status_bar(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    segments: &[String],
) {
    let message = segments.join("  ");
    let paragraph = Paragraph::new(message)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}
