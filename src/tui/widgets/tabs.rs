use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render_tabs(frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let titles = vec!["Cache", "Apps", "Temp", "Logs", "Packages", "Kernels"];
    let list = List::new(titles.iter().map(|t| ListItem::new(*t)))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(
        list.block(Block::default().title("Categories").borders(Borders::ALL)),
        area,
    );
}
