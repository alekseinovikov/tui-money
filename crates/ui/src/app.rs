use std::io;

use ratatui::Frame;

use crate::event::Action;
use crate::screens::{DashboardScreen, ScreenId};

pub struct App {
    active: ScreenId,
    dashboard: DashboardScreen,
}

impl App {
    pub fn new() -> Self {
        Self {
            active: ScreenId::Dashboard,
            dashboard: DashboardScreen::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) {
        match self.active {
            ScreenId::Dashboard => self.dashboard.render(frame),
        }
    }

    pub fn apply(&mut self, action: Action) -> io::Result<bool> {
        match action {
            Action::Quit => Ok(true),
            Action::None => Ok(false),
            Action::Go(screen) => {
                self.active = screen;
                Ok(false)
            }
        }
    }
}
