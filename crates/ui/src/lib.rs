mod app;
mod event;
mod layout;
mod screens;
mod widgets;

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event as ct_event;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::App;
use crate::event::handle_event;

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    loop {
        terminal.draw(|frame| app.render(frame))?;

        if ct_event::poll(Duration::from_millis(100))? {
            let evt = ct_event::read()?;
            let action = handle_event(&evt);
            if app.apply(action)? {
                break;
            }
        }
    }

    Ok(())
}
