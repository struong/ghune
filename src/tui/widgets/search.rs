use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::{AppMode, AppState};

pub fn render_search(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_active = state.mode == AppMode::Search;

    let border_style = if is_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            " Search ",
            if is_active {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));

    let cursor_char = if is_active { "_" } else { "" };

    let content = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Yellow)),
        Span::raw(&state.search_query),
        Span::styled(cursor_char, Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK)),
    ]);

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}
