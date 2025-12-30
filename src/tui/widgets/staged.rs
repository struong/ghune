use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::state::{AppMode, AppState};

pub fn render_staged(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_active = state.mode == AppMode::Staging;

    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let staged_count = state.staged_for_deletion.len();
    let title = format!(" Staged ({}) ", staged_count);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            title,
            if is_active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if staged_count > 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));

    let staged_names = state.staged_repos_sorted();

    let items: Vec<ListItem> = staged_names
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let is_selected = idx == state.staged_selected_index && is_active;

            let repo = state.repositories.iter().find(|r| &r.full_name == name);

            let mut spans = Vec::new();

            spans.push(Span::styled("■ ", Style::default().fg(Color::Red)));

            let name_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::Red)
            };

            spans.push(Span::styled(name.as_str(), name_style));

            if let Some(repo) = repo {
                spans.push(Span::raw(" "));
                if repo.private {
                    spans.push(Span::styled("🔒", Style::default()));
                }
                if repo.fork {
                    spans.push(Span::styled("🍴", Style::default()));
                }
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(block);

    let mut list_state = ListState::default();
    if is_active && !staged_names.is_empty() {
        list_state.select(Some(state.staged_selected_index));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}
