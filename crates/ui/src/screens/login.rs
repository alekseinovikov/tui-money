use domain::EntryRepository;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::{Screen, ScreenId, ScreenResult};
use crate::event::Action;
use crate::layout::centered_rect;

pub struct LoginScreen {
    focus: LoginFocus,
    user_options: Vec<String>,
    user_selected: usize,
    user_dropdown_open: bool,
    password_input: String,
}

impl LoginScreen {
    pub fn new() -> Self {
        Self {
            focus: LoginFocus::User,
            user_options: vec![
                "alice".to_string(),
                "bob".to_string(),
                "charlie".to_string(),
            ],
            user_selected: 0,
            user_dropdown_open: false,
            password_input: String::new(),
        }
    }

    fn focus_next(&mut self) {
        if self.user_dropdown_open {
            return; // Lock focus when dropdown is open
        }
        self.focus = match self.focus {
            LoginFocus::User => LoginFocus::Password,
            LoginFocus::Password => LoginFocus::LoginButton,
            LoginFocus::LoginButton => LoginFocus::CreateUserButton,
            LoginFocus::CreateUserButton => LoginFocus::User,
        };
    }

    fn focus_prev(&mut self) {
        if self.user_dropdown_open {
            return; // Lock focus when dropdown is open
        }
        self.focus = match self.focus {
            LoginFocus::User => LoginFocus::CreateUserButton,
            LoginFocus::Password => LoginFocus::User,
            LoginFocus::LoginButton => LoginFocus::Password,
            LoginFocus::CreateUserButton => LoginFocus::LoginButton,
        };
    }

    fn activate(&mut self) -> ScreenResult {
        match self.focus {
            LoginFocus::User => {
                self.user_dropdown_open = !self.user_dropdown_open;
                ScreenResult::None
            }
            LoginFocus::CreateUserButton => ScreenResult::Go(ScreenId::CreateUser),
            LoginFocus::LoginButton => {
                // TODO: Verify password etc. For now just go to dashboard.
                ScreenResult::Go(ScreenId::Dashboard)
            }
            LoginFocus::Password => {
                // Allow pressing Enter in password field to submit
                ScreenResult::Go(ScreenId::Dashboard)
            }
        }
    }

    fn selected_user_label(&self) -> String {
        self.user_options
            .get(self.user_selected)
            .cloned()
            .unwrap_or_else(|| "Select user".to_string())
    }

    fn dropdown_lines(&self) -> Vec<Line<'_>> {
        let mut lines = Vec::new();
        let max_items = self.user_options.len().min(4);
        // show a simple scrollable view or just top 4? simpler: just top 4
        // ideally we would implement a real scroll, but for now we clamp
        for (idx, name) in self.user_options.iter().enumerate().take(max_items) {
            let style = if idx == self.user_selected {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            lines.push(Line::from(Span::styled(format!("  {}", name), style)));
        }
        lines
    }
}

impl Screen for LoginScreen {
    fn render(&mut self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let dropdown_height = if self.user_dropdown_open {
            self.user_options.len().min(4) as u16
        } else {
            0
        };
        // Form structure
        let form_height = 10; // fixed height for the main form
        let form_area = centered_rect(area, 50, form_height);

        let block = Block::default()
            .title(" Login System ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        frame.render_widget(block.clone(), form_area);

        let inner_area = block.inner(form_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // User
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Password
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Buttons
            ])
            .split(inner_area);

        // Styles
        let focused_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let default_style = Style::default().fg(Color::Gray);

        let user_style = if self.focus == LoginFocus::User {
            focused_style
        } else {
            default_style
        };
        let pass_style = if self.focus == LoginFocus::Password {
            focused_style
        } else {
            default_style
        };
        let login_btn_style = if self.focus == LoginFocus::LoginButton {
            focused_style.bg(Color::Blue).fg(Color::White)
        } else {
            default_style
        };
        let create_btn_style = if self.focus == LoginFocus::CreateUserButton {
            focused_style.bg(Color::Blue).fg(Color::White)
        } else {
            default_style
        };

        // 1. User Field
        let user_arrow = if self.user_dropdown_open {
            "▲"
        } else {
            "▼"
        };
        let user_line = Line::from(vec![
            Span::raw("Username: "),
            Span::styled(
                format!("{} {}", self.selected_user_label(), user_arrow),
                user_style,
            ),
        ]);
        frame.render_widget(Paragraph::new(user_line), chunks[0]);

        // 2. Password Field
        let pass_stars = "*".repeat(self.password_input.len());
        let pass_line = Line::from(vec![
            Span::raw("Password: "),
            Span::styled(pass_stars.to_string(), pass_style),
            if (frame.count() / 30).is_multiple_of(2) {
                Span::raw("_")
            } else {
                Span::raw(" ")
            }, // blinking cursor simulation
        ]);
        frame.render_widget(Paragraph::new(pass_line), chunks[2]);

        // 3. Buttons
        let btns = Line::from(vec![
            Span::styled("[ Login ]", login_btn_style),
            Span::raw("   "),
            Span::styled("[ Create User ]", create_btn_style),
        ]);
        frame.render_widget(
            Paragraph::new(btns).alignment(ratatui::layout::Alignment::Center),
            chunks[4],
        );

        // Dropdown Overlay
        if self.user_dropdown_open {
            let dropdown_area = ratatui::layout::Rect {
                x: chunks[0].x + 10,
                y: chunks[0].y + 1,
                width: 30,
                height: dropdown_height + 2,
            };
            frame.render_widget(Clear, dropdown_area);
            let drop_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray));
            let drop_inner = drop_block.inner(dropdown_area);
            frame.render_widget(drop_block, dropdown_area);

            let items = self.dropdown_lines();
            frame.render_widget(Paragraph::new(items), drop_inner);
        }
    }

    fn handle_action(&mut self, action: Action, _repo: &mut dyn EntryRepository) -> ScreenResult {
        match action {
            Action::Quit => ScreenResult::Quit,
            Action::Cancel => {
                if self.user_dropdown_open {
                    self.user_dropdown_open = false;
                }
                ScreenResult::None
            }
            Action::FocusNext => {
                self.focus_next();
                ScreenResult::None
            }
            Action::FocusPrev => {
                self.focus_prev();
                ScreenResult::None
            }
            Action::NavUp => {
                if self.user_dropdown_open && self.user_selected > 0 {
                    self.user_selected -= 1;
                }
                ScreenResult::None
            }
            Action::NavDown => {
                if self.user_dropdown_open && self.user_selected + 1 < self.user_options.len() {
                    self.user_selected += 1;
                }
                ScreenResult::None
            }
            Action::Activate => self.activate(),
            Action::InputChar(ch) => {
                if self.focus == LoginFocus::Password {
                    self.password_input.push(ch);
                }
                ScreenResult::None
            }
            Action::Backspace => {
                if self.focus == LoginFocus::Password {
                    self.password_input.pop();
                }
                ScreenResult::None
            }
            _ => ScreenResult::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoginFocus {
    User,
    Password,
    LoginButton,
    CreateUserButton,
}
