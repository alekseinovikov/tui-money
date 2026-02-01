mod dashboard;

pub use dashboard::DashboardScreen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    Dashboard,
}

pub trait Screen {
    fn render(&mut self, frame: &mut ratatui::Frame<'_>);
}
