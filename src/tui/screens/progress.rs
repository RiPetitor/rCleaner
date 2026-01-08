use crate::tui::screens::common::render_header;
use crate::tui::state::State;
use crate::tui::widgets::progress_bar::render_progress_bar;
use crate::tui::widgets::status_bar::render_status_bar;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

pub fn render_progress_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &State,
    system_label: &str,
) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Cleaning");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(inner);

    render_header(frame, chunks[0], system_label, state.safety_level);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[1]);

    render_progress_bar(frame, body[0], state.cleanup_progress, "Progress");

    let step_text = state
        .cleanup_step
        .as_deref()
        .unwrap_or("Working on cleanup...");
    let step =
        Paragraph::new(step_text).block(Block::default().borders(Borders::ALL).title("Step"));
    frame.render_widget(step, body[1]);

    let keys = vec!["Cleaning in progress...".to_string()];
    render_status_bar(frame, chunks[2], &keys);
}
