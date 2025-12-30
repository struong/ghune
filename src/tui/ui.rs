use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::state::AppState;

use super::widgets::{
    render_header, render_repo_list, render_search, render_staged, render_status,
};

pub fn render(frame: &mut Frame, state: &AppState) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(frame.area());

    render_header(frame, main_chunks[0], state);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(content_chunks[0]);

    render_search(frame, left_chunks[0], state);
    render_repo_list(frame, left_chunks[1], state);
    render_staged(frame, content_chunks[1], state);
    render_status(frame, main_chunks[2], state);
}
