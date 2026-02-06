use std::io;

use ratatui::Frame;

use crate::event::Action;
use crate::screens::{
    CreateUserScreen, DashboardScreen, LoginScreen, Screen, ScreenId, ScreenResult,
};
use domain::EntryRepository;

pub struct App {
    should_quit: bool,
    active_screen_id: ScreenId,
    dashboard: DashboardScreen,
    login: LoginScreen,
    create_user: CreateUserScreen,
    repo: Box<dyn EntryRepository>,
}

impl App {
    pub fn new(repo: Box<dyn EntryRepository>) -> Self {
        Self {
            should_quit: false,
            active_screen_id: ScreenId::Login,
            dashboard: DashboardScreen::new(),
            login: LoginScreen::new(),
            create_user: CreateUserScreen::new(),
            repo,
        }
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) {
        match self.active_screen_id {
            ScreenId::Dashboard => self.dashboard.render(frame),
            ScreenId::Login => self.login.render(frame),
            ScreenId::CreateUser => self.create_user.render(frame),
        }
    }

    pub fn apply(&mut self, action: Action) -> io::Result<bool> {
        let repo = &mut *self.repo;
        let result = match self.active_screen_id {
            ScreenId::Dashboard => self.dashboard.handle_action(action, repo),
            ScreenId::Login => self.login.handle_action(action, repo),
            ScreenId::CreateUser => self.create_user.handle_action(action, repo),
        };

        match result {
            ScreenResult::Quit => self.should_quit = true,
            ScreenResult::Go(id) => self.switch_screen(id),
            ScreenResult::None => {}
        }

        Ok(self.should_quit)
    }

    fn switch_screen(&mut self, id: ScreenId) {
        self.active_screen_id = id;
        let repo = &mut *self.repo;
        let _ = match self.active_screen_id {
            ScreenId::Dashboard => self.dashboard.init(repo),
            ScreenId::Login => self.login.init(repo),
            ScreenId::CreateUser => self.create_user.init(repo),
        };
    }
}
