use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::state::{AppMode, AppState};

pub fn render_staged(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_active = state.mode == AppMode::Staging || state.mode == AppMode::ConfirmDeletion;

    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let staged_count = state.staged_for_deletion.len();
    let tab_hint = if !is_active { "[Tab] " } else { "" };
    let title = format!(" {}Staged ({}) ", tab_hint, staged_count);

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
            let is_selected = idx == state.staged_selected_index && state.mode == AppMode::Staging;

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
    if state.mode == AppMode::Staging && !staged_names.is_empty() {
        list_state.select(Some(state.staged_selected_index));
    }

    frame.render_stateful_widget(list, area, &mut list_state);

    if state.mode == AppMode::ConfirmDeletion {
        render_confirmation_dialog(frame, area, state);
    }
}

fn render_confirmation_dialog(frame: &mut Frame, area: Rect, state: &AppState) {
    let dialog_width = 45u16;
    let dialog_height = 7u16;

    let x = area.x + area.width.saturating_sub(dialog_width) / 2;
    let y = area.y + area.height.saturating_sub(dialog_height) / 2;
    let dialog_area = Rect::new(x, y, dialog_width.min(area.width), dialog_height.min(area.height));

    frame.render_widget(Clear, dialog_area);

    let count = state.staged_for_deletion.len();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(Span::styled(
            " Confirm Deletion ",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let prompt = Line::from(vec![
        Span::raw("Type "),
        Span::styled(
            count.to_string(),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to delete "),
        Span::styled(
            format!("{} repo{}", count, if count == 1 { "" } else { "s" }),
            Style::default().fg(Color::Red),
        ),
        Span::raw(":"),
    ]);
    frame.render_widget(Paragraph::new(prompt), chunks[0]);

    let input_line = Line::from(vec![
        Span::raw("> "),
        Span::styled(
            &state.confirmation_input,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("_", Style::default().fg(Color::Gray)),
    ]);
    frame.render_widget(Paragraph::new(input_line), chunks[1]);

    let hint = Line::from(Span::styled(
        "Press Esc to cancel",
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(hint), chunks[2]);
}
