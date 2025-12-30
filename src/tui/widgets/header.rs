use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::{AppMode, AppState};

pub fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let mode_style = match state.mode {
        AppMode::Search => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        AppMode::Staging => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        AppMode::Deleting => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    };

    let mode_text = match state.mode {
        AppMode::Search => "SEARCH",
        AppMode::Staging => "STAGING",
        AppMode::Deleting => "DELETING",
    };

    let repo_count = if state.loading {
        "Loading...".to_string()
    } else {
        format!("Repos: {}", state.repositories.len())
    };

    let dry_run_indicator = if state.dry_run {
        Span::styled(
            " [DRY RUN]",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
    };

    let line = Line::from(vec![
        Span::styled(
            "prune",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        dry_run_indicator,
        Span::raw("  "),
        Span::styled(format!("[{}]", mode_text), mode_style),
        Span::raw("  "),
        Span::styled(repo_count, Style::default().fg(Color::Gray)),
    ]);

    let header = Paragraph::new(line);
    frame.render_widget(header, area);
}
