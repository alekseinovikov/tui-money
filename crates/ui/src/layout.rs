use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn main_chunks(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(area)
        .to_vec()
}
