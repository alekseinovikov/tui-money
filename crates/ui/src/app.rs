use std::io;

use ratatui::Frame;

use crate::event::Action;
use crate::screens::{CreateUserScreen, DashboardScreen, LoginScreen, ScreenId};

pub struct App {
    active: ScreenId,
    dashboard: DashboardScreen,
    login: LoginScreen,
    create_user: CreateUserScreen,
}

impl App {
    pub fn new() -> Self {
        Self {
            active: ScreenId::Login,
            dashboard: DashboardScreen::new(),
            login: LoginScreen::new(),
            create_user: CreateUserScreen::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) {
        match self.active {
            ScreenId::Dashboard => self.dashboard.render(frame),
            ScreenId::Login => self.login.render(frame),
            ScreenId::CreateUser => self.create_user.render(frame),
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
            Action::FocusNext => {
                if self.active == ScreenId::Login {
                    self.login.focus_next();
                } else if self.active == ScreenId::CreateUser {
                    self.create_user.focus_next();
                }
                Ok(false)
            }
            Action::FocusPrev => {
                if self.active == ScreenId::Login {
                    self.login.focus_prev();
                } else if self.active == ScreenId::CreateUser {
                    self.create_user.focus_prev();
                }
                Ok(false)
            }
            Action::Activate => {
                match self.active {
                    ScreenId::Login => {
                        if let Some(screen) = self.login.activate_or_toggle() {
                            self.active = screen;
                        }
                    }
                    ScreenId::CreateUser => {
                        if self.create_user.activate() {
                            self.active = ScreenId::Login;
                        }
                    }
                    _ => {}
                }
                Ok(false)
            }
            Action::InputChar(ch) => {
                match self.active {
                    ScreenId::Login => self.login.input_char(ch),
                    ScreenId::CreateUser => self.create_user.input_char(ch),
                    _ => {}
                }
                Ok(false)
            }
            Action::Backspace => {
                match self.active {
                    ScreenId::Login => self.login.backspace(),
                    ScreenId::CreateUser => self.create_user.backspace(),
                    _ => {}
                }
                Ok(false)
            }
            Action::NavUp => {
                match self.active {
                    ScreenId::Login => self.login.nav_up(),
                    ScreenId::CreateUser => self.create_user.nav_up(),
                    _ => {}
                }
                Ok(false)
            }
            Action::NavDown => {
                match self.active {
                    ScreenId::Login => self.login.nav_down(),
                    ScreenId::CreateUser => self.create_user.nav_down(),
                    _ => {}
                }
                Ok(false)
            }
            Action::NavLeft => {
                match self.active {
                    ScreenId::Login => self.login.nav_left(),
                    ScreenId::CreateUser => self.create_user.nav_left(),
                    _ => {}
                }
                Ok(false)
            }
            Action::NavRight => {
                match self.active {
                    ScreenId::Login => self.login.nav_right(),
                    ScreenId::CreateUser => self.create_user.nav_right(),
                    _ => {}
                }
                Ok(false)
            }
        }
    }
}
