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
    username_input: String,
    password_input: String,
    error_message: Option<String>,
}

impl LoginScreen {
    pub fn new() -> Self {
        Self {
            focus: LoginFocus::User,
            user_options: Vec::new(),
            user_selected: 0,
            user_dropdown_open: false,
            username_input: String::new(),
            password_input: String::new(),
            error_message: None,
        }
    }

    fn focus_next(&mut self) {
        if self.user_dropdown_open {
            return;
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
            return;
        }
        self.focus = match self.focus {
            LoginFocus::User => LoginFocus::CreateUserButton,
            LoginFocus::Password => LoginFocus::User,
            LoginFocus::LoginButton => LoginFocus::Password,
            LoginFocus::CreateUserButton => LoginFocus::LoginButton,
        };
    }

    fn activate(&mut self, repo: &mut dyn EntryRepository) -> ScreenResult {
        match self.focus {
            LoginFocus::User => {
                // Toggle dropdown
                if self.user_dropdown_open {
                    self.user_dropdown_open = false;
                } else {
                    // Load users if needed
                     if let Ok(users) = repo.list_users() {
                         self.user_options = users;
                     }
                    self.user_dropdown_open = true;
                }
                ScreenResult::None
            }
            LoginFocus::CreateUserButton => {
                if self.username_input.trim().is_empty() || self.password_input.is_empty() {
                    self.error_message = Some("Username and password required".to_string());
                    return ScreenResult::None;
                }
                match repo.create_user(&self.username_input, &self.password_input) {
                    Ok(_) => {
                         self.error_message = Some("User created! Log in now.".to_string());
                         // Clear password to force re-entry or just login? Safe to generic message.
                         self.password_input.clear();
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Error: {}", e));
                    }
                }
                ScreenResult::None
            }
            LoginFocus::LoginButton => {
                 self.perform_login(repo)
            }
            LoginFocus::Password => {
                self.perform_login(repo)
            }
        }
    }

    fn perform_login(&mut self, repo: &dyn EntryRepository) -> ScreenResult {
         // Using "GlobalEntryRepo" aliases just dyn EntryRepository for brevity in thought, 
         // but here we use the trait directly.
         if self.username_input.trim().is_empty() {
             self.error_message = Some("Username required".to_string());
             return ScreenResult::None;
         }
         match repo.verify_user(&self.username_input, &self.password_input) {
             Ok(Some(_user)) => {
                 ScreenResult::Go(ScreenId::Dashboard)
             }
             Ok(None) => {
                 self.error_message = Some("Invalid credentials".to_string());
                 ScreenResult::None
             }
             Err(e) => {
                 self.error_message = Some(format!("Error: {}", e));
                 ScreenResult::None
             }
         }
    }

    fn dropdown_lines(&self) -> Vec<Line<'_>> {
        let mut lines = Vec::new();
        let max_items = self.user_options.len().min(4);
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
        let form_height = 12; // increased for error msg
        let form_area = centered_rect(area, 60, form_height);

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
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Error
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
        // Use username_input
        let user_display = if self.username_input.is_empty() {
            "Type or Select..."
        } else {
            &self.username_input
        };
        
        let user_line = Line::from(vec![
            Span::raw("Username: "),
            Span::styled(
                format!("{} {}", user_display, user_arrow),
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
            }, 
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
        
        // 4. Error Message
        if let Some(err) = &self.error_message {
            let err_line = Line::from(Span::styled(err, Style::default().fg(Color::Red)));
            frame.render_widget(Paragraph::new(err_line).alignment(ratatui::layout::Alignment::Center), chunks[6]);
        }

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

    fn handle_action(&mut self, action: Action, repo: &mut dyn EntryRepository) -> ScreenResult {
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
            Action::Activate => {
                if self.user_dropdown_open {
                    // Selection confirmed
                    if let Some(name) = self.user_options.get(self.user_selected) {
                        self.username_input = name.clone();
                    }
                    self.user_dropdown_open = false;
                    ScreenResult::None
                } else {
                    self.activate(repo)
                }
            },
            Action::InputChar(ch) => {
                self.error_message = None;
                if self.focus == LoginFocus::User {
                     self.username_input.push(ch);
                } else if self.focus == LoginFocus::Password {
                    self.password_input.push(ch);
                }
                ScreenResult::None
            }
            Action::Backspace => {
                self.error_message = None;
                if self.focus == LoginFocus::User {
                     self.username_input.pop();
                } else if self.focus == LoginFocus::Password {
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
