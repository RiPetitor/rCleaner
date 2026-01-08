use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Tabs};

pub const TAB_TITLES: [&str; 6] = ["Cache", "Apps", "Temp", "Logs", "Packages", "Kernels"];

pub fn render_tabs(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, active: usize) {
    let titles = TAB_TITLES
        .iter()
        .map(|title| Line::from(format!(" {title} ")));

    let tabs = Tabs::new(titles)
        .select(active)
        .block(Block::default().borders(Borders::ALL).title("Categories"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::White))
        .divider("|");

    frame.render_widget(tabs, area);
}
