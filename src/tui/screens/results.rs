pub fn render_results_screen(frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    use ratatui::widgets::Paragraph;

    let paragraph = Paragraph::new("Results screen implementation")
        .block(ratatui::widgets::Block::default().title("Cleanup Results"));

    frame.render_widget(paragraph, area);
}
