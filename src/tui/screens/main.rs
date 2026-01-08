pub fn render_main_screen(frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    use ratatui::widgets::Paragraph;

    let paragraph = Paragraph::new("Main screen implementation")
        .block(ratatui::widgets::Block::default().title("Main Screen"));

    frame.render_widget(paragraph, area);
}
