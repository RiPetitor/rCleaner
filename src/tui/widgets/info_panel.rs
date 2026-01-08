use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render_info_panel(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, info: &str) {
    let paragraph = Paragraph::new(info)
        .block(Block::default().title("Information").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}
