pub fn render_settings_screen(frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    use ratatui::widgets::Paragraph;

    let paragraph = Paragraph::new("Settings screen implementation")
        .block(ratatui::widgets::Block::default().title("Settings"));

    frame.render_widget(paragraph, area);
}
