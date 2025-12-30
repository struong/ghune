use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::{AppMode, AppState, SortMode, StatusLevel};

pub fn render_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let keybindings = match state.mode {
        AppMode::Search => vec![
            ("C-j/k", "Nav"),
            ("Enter", "Stage"),
            ("C-t", "Staging"),
            ("C-p", "Private"),
            ("C-f", "Forks"),
            ("C-s", "Sort"),
            ("C-c", "Quit"),
        ],
        AppMode::Staging => vec![
            ("C-j/C-k", "Navigate"),
            ("Backspace", "Unstage"),
            ("Enter", "Delete"),
            ("C-t/Esc", "Back"),
            ("C-c/q", "Quit"),
        ],
        AppMode::Deleting => vec![("", "Deleting repositories...")],
    };

    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, action)) in keybindings.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" │ "));
        }
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(": "));
        spans.push(Span::styled(*action, Style::default().fg(Color::Gray)));
    }

    // Add active filters/sort indicators
    if state.mode == AppMode::Search {
        let mut filters = Vec::new();
        if state.filter_private {
            filters.push("🔒");
        }
        if state.filter_forks {
            filters.push("🍴");
        }
        if state.sort_mode != SortMode::LastUpdated {
            filters.push(state.sort_mode.label());
        }
        if !filters.is_empty() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                format!("[{}]", filters.join(" ")),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));
        }
    }

    let keybindings_line = Line::from(spans);

    let status_line = if let Some((current, total)) = state.deletion_progress {
        let spinner = state.spinner();
        let repo_name = state.deleting_repo.as_deref().unwrap_or("");
        let dry_run_prefix = if state.dry_run { "[DRY RUN] " } else { "" };
        Line::from(vec![
            Span::styled(
                format!("{} ", spinner),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}Deleting {}/{}: ", dry_run_prefix, current, total),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                repo_name,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else if let Some((ref msg, ref level)) = state.status_message {
        let style = match level {
            StatusLevel::Info => Style::default().fg(Color::Blue),
            StatusLevel::Warning => Style::default().fg(Color::Yellow),
            StatusLevel::Error => Style::default().fg(Color::Red),
            StatusLevel::Success => Style::default().fg(Color::Green),
        };
        Line::from(Span::styled(msg.as_str(), style))
    } else {
        Line::from(Span::raw(""))
    };

    let paragraph = Paragraph::new(vec![keybindings_line, status_line]);
    frame.render_widget(paragraph, area);
}
