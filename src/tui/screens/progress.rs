pub fn render_progress_screen(frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    use ratatui::widgets::Paragraph;

    let paragraph = Paragraph::new("Progress screen implementation")
        .block(ratatui::widgets::Block::default().title("Cleanup Progress"));

    frame.render_widget(paragraph, area);
}
