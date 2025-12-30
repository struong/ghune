use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::state::{AppMode, AppState};

fn format_relative_time(updated_at: &Option<String>) -> String {
    let Some(date_str) = updated_at else {
        return String::new();
    };

    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) else {
        return String::new();
    };

    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(dt);

    let days = duration.num_days();
    if days < 1 {
        let hours = duration.num_hours();
        if hours < 1 {
            return "now".to_string();
        }
        return format!("{}h", hours);
    }
    if days < 30 {
        return format!("{}d", days);
    }
    if days < 365 {
        return format!("{}mo", days / 30);
    }
    format!("{}y", days / 365)
}

pub fn render_repo_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_active = state.mode == AppMode::Search;

    let border_style = if is_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let filtered_count = state.filtered_indices.len();
    let total_count = state.repositories.len();

    let title = if state.search_query.is_empty() {
        format!(" Repositories ({}) ", total_count)
    } else {
        format!(" Repositories ({}/{}) ", filtered_count, total_count)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            title,
            if is_active {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));

    let items: Vec<ListItem> = state
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(display_idx, &repo_idx)| {
            let repo = &state.repositories[repo_idx];
            let is_selected = display_idx == state.selected_index && is_active;
            let is_staged = state.staged_for_deletion.contains(&repo.full_name);

            let mut spans = Vec::new();

            if is_staged {
                spans.push(Span::styled("● ", Style::default().fg(Color::Red)));
            } else {
                spans.push(Span::raw("  "));
            }

            let name_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Green)
            } else if is_staged {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };

            spans.push(Span::styled(&repo.full_name, name_style));
            spans.push(Span::raw(" "));

            if repo.private {
                spans.push(Span::styled("🔒", Style::default()));
            }
            if repo.fork {
                spans.push(Span::styled("🍴", Style::default()));
            }

            if repo.stargazers_count > 0 {
                spans.push(Span::styled(
                    format!(" ★{}", repo.stargazers_count),
                    Style::default().fg(Color::Yellow),
                ));
            }

            let relative_time = format_relative_time(&repo.updated_at);
            if !relative_time.is_empty() {
                spans.push(Span::styled(
                    format!(" {}", relative_time),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(block);

    let mut list_state = ListState::default();
    if is_active && !state.filtered_indices.is_empty() {
        list_state.select(Some(state.selected_index));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}
