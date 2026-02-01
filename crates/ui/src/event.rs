use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::screens::ScreenId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Go(ScreenId),
}

pub fn handle_event(event: &Event) -> Action {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('d') => Action::Go(ScreenId::Dashboard),
            _ => Action::None,
        },
        _ => Action::None,
    }
}
