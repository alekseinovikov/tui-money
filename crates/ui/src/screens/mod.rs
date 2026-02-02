mod dashboard;
mod create_user;
mod login;

pub use dashboard::DashboardScreen;
pub use create_user::CreateUserScreen;
pub use login::LoginScreen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    Dashboard,
    Login,
    CreateUser,
}

pub trait Screen {
    fn render(&mut self, frame: &mut ratatui::Frame<'_>);
}
