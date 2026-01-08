pub fn render_cleanup_screen(frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    use ratatui::widgets::Paragraph;

    let paragraph = Paragraph::new("Cleanup screen implementation")
        .block(ratatui::widgets::Block::default().title("Cleanup"));

    frame.render_widget(paragraph, area);
}
