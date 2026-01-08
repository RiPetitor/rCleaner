use crate::tui::screens::common::render_header;
use crate::tui::state::State;
use crate::tui::widgets::status_bar::render_status_bar;
use crate::utils::size_format::format_size;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};

pub fn render_results_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &State,
    system_label: &str,
) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Results");
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
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(chunks[1]);

    let summary_text = match &state.last_result {
        Some(result) => format!(
            "Cleaned: {}\nSkipped: {}\nFreed: {}\nErrors: {}",
            result.cleaned_items,
            result.skipped_items,
            format_size(result.freed_bytes),
            result.errors.len()
        ),
        None => "No cleanup results available.".to_string(),
    };
    let summary =
        Paragraph::new(summary_text).block(Block::default().borders(Borders::ALL).title("Summary"));
    frame.render_widget(summary, body[0]);

    let error_items = match &state.last_result {
        Some(result) if !result.errors.is_empty() => result
            .errors
            .iter()
            .take(body[1].height.saturating_sub(2) as usize)
            .map(|err| ListItem::new(err.clone()))
            .collect(),
        _ => vec![ListItem::new("No errors reported.")],
    };

    let errors = List::new(error_items)
        .block(Block::default().borders(Borders::ALL).title("Errors"))
        .style(Style::default().fg(Color::White));
    frame.render_widget(errors, body[1]);

    let keys = vec!["[Enter] Back".to_string(), "[Esc] Back".to_string()];
    render_status_bar(frame, chunks[2], &keys);
}
