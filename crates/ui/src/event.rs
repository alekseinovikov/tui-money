use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::screens::ScreenId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Go(ScreenId),
    FocusNext,
    FocusPrev,
    Activate,
    InputChar(char),
    Backspace,
    NavUp,
    NavDown,
    NavLeft,
    NavRight,
}

pub fn handle_event(event: &Event) -> Action {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Tab => Action::FocusNext,
            KeyCode::BackTab => Action::FocusPrev,
            KeyCode::Up => Action::NavUp,
            KeyCode::Down => Action::NavDown,
            KeyCode::Left => Action::NavLeft,
            KeyCode::Right => Action::NavRight,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Enter => Action::Activate,
            KeyCode::Char(ch) if key.modifiers.is_empty() => Action::InputChar(ch),
            _ => Action::None,
        },
        _ => Action::None,
    }
}
