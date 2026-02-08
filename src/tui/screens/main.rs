use crate::i18n;
use crate::tui::screens::common::{Theme, render_header, styled_block};
use crate::tui::state::State;
use crate::tui::widgets::info_panel::render_info_panel;
use crate::tui::widgets::selectable_list::render_selectable_list;
use crate::tui::widgets::status_bar::render_status_bar;
use crate::tui::widgets::tabs::render_tabs;
use crate::utils::size_format::{format_percentage, format_size};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

pub fn render_main_screen(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &State,
    system_label: &str,
) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Theme::BORDER));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(inner);

    render_header(frame, chunks[0], system_label, state.safety_level);
    render_tabs(frame, chunks[1], state);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(chunks[2]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(body_chunks[0]);

    let visible_items = state.visible_items();
    if visible_items.is_empty() {
        let empty_msg = if state.status_message.as_deref() == Some(i18n::scanning()) {
            i18n::scanning()
        } else {
            i18n::no_items()
        };
        let empty = Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(empty_msg, Style::default().fg(Theme::TEXT_MUTED)),
        ]))
        .block(styled_block(i18n::items()));
        frame.render_widget(empty, left_chunks[0]);
    } else {
        render_selectable_list(
            frame,
            left_chunks[0],
            &format!("{} ({})", i18n::items(), visible_items.len()),
            &visible_items,
            state.selected_index,
        );
    }

    let matches = visible_items.len();
    render_search_box(frame, left_chunks[1], state, matches);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(0)])
        .split(body_chunks[1]);

    let info_text = match state.selected_item() {
        Some(item) => {
            let status = if item.can_clean {
                (if item.selected {
                    i18n::status_selected()
                } else {
                    i18n::status_available()
                })
                .to_string()
            } else if let Some(reason) = &item.blocked_reason {
                format!("{}: {reason}", i18n::status_blocked())
            } else {
                format!("{}: {}", i18n::status_blocked(), i18n::safety_rules())
            };

            let deps = if item.dependencies.is_empty() {
                String::new()
            } else {
                format!("\nDeps: {}", item.dependencies.join(", "))
            };

            format!(
                "{}\n{}\n{}\n{}{}\n\n{}",
                item.name,
                format_size(item.size),
                format_source(item),
                status,
                deps,
                item.description
            )
        }
        None => i18n::select_to_see().to_string(),
    };
    render_info_panel(frame, right_chunks[0], i18n::details(), &info_text);

    let selected_pct = format_percentage(state.selected_size, state.total_size);
    let summary_text = format!(
        "{}: {} ({})\n{}: {} ({})\n\n{}: {}\n{}: {}",
        i18n::selected_label(),
        state.selected_count(),
        selected_pct,
        i18n::size_label(),
        format_size(state.selected_size),
        selected_pct,
        i18n::total_label(),
        state.items.len(),
        i18n::size_label(),
        format_size(state.total_size)
    );
    render_info_panel(frame, right_chunks[1], i18n::summary(), &summary_text);

    let mut keys: Vec<String> = vec![
        i18n::key_category().into(),
        i18n::key_select().into(),
        i18n::key_all().into(),
        i18n::key_clean().into(),
        i18n::key_settings().into(),
        i18n::key_refresh().into(),
        i18n::key_search().into(),
        i18n::key_quit().into(),
    ];

    if let Some(message) = state.status_message.as_deref() {
        keys.push(message.to_string());
    }

    render_status_bar(frame, chunks[3], &keys);
}

fn format_source(item: &crate::models::CleanupItem) -> String {
    match &item.source {
        crate::models::CleanupSource::FileSystem => i18n::source_fs().to_string(),
        crate::models::CleanupSource::PackageManager(name) => format!("Source: {name}"),
        crate::models::CleanupSource::Container(name) => format!("Source: {name}"),
    }
}

fn render_search_box(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &State,
    matches: usize,
) {
    let border_color = if state.search_active {
        Theme::BORDER_ACTIVE
    } else {
        Theme::BORDER
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(" {} ", i18n::search()),
            Style::default()
                .fg(if state.search_active {
                    Theme::ACCENT
                } else {
                    Theme::TEXT_DIM
                })
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let query = state.search_query.trim();
    let search_icon = if state.search_active { "/ " } else { "  " };
    let label = if query.is_empty() {
        format!("{search_icon}{}", i18n::type_to_search())
    } else {
        format!("{search_icon}{}", state.search_query)
    };

    let line = Line::from(vec![
        Span::styled(
            label,
            if query.is_empty() && !state.search_active {
                Style::default().fg(Theme::TEXT_MUTED)
            } else {
                Style::default().fg(Theme::TEXT)
            },
        ),
        Span::raw("  "),
        Span::styled(
            format!("{matches} {}", i18n::matches_label()),
            Style::default().fg(Theme::TEXT_MUTED),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), inner);

    if state.search_active {
        let cursor_offset = 2 + state.search_query.len();
        let max_x = inner.width.saturating_sub(1) as usize;
        let cursor_x = inner.x + cursor_offset.min(max_x) as u16;
        frame.set_cursor_position(ratatui::layout::Position {
            x: cursor_x,
            y: inner.y,
        });
    }
}
