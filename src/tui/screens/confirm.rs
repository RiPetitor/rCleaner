pub fn render_confirm_screen(frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    use ratatui::widgets::Paragraph;

    let paragraph = Paragraph::new("Confirm screen implementation")
        .block(ratatui::widgets::Block::default().title("Confirm Cleanup"));

    frame.render_widget(paragraph, area);
}
