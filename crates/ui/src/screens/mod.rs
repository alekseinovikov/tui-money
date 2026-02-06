mod create_user;
mod dashboard;
mod login;

pub use create_user::CreateUserScreen;
pub use dashboard::DashboardScreen;
pub use login::LoginScreen;

use crate::event::Action;
use domain::EntryRepository;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    Dashboard,
    Login,
    CreateUser,
}

pub enum ScreenResult {
    None,
    Quit,
    Go(ScreenId),
}

pub trait Screen {
    fn init(&mut self, _repo: &mut dyn EntryRepository) -> Result<(), domain::DomainError> {
        Ok(())
    }
    fn render(&mut self, frame: &mut ratatui::Frame<'_>);
    fn handle_action(&mut self, action: Action, repo: &mut dyn EntryRepository) -> ScreenResult;
}
