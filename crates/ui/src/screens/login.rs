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
        self.user_dropdown_open = false;
        self.focus = match self.focus {
            LoginFocus::User => LoginFocus::Password,
            LoginFocus::Password => LoginFocus::LoginButton,
            LoginFocus::LoginButton => LoginFocus::CreateUserButton,
            LoginFocus::CreateUserButton => LoginFocus::User,
        };
    }

    fn focus_prev(&mut self) {
        self.user_dropdown_open = false;
        self.focus = match self.focus {
            LoginFocus::User => LoginFocus::CreateUserButton,
            LoginFocus::Password => LoginFocus::User,
            LoginFocus::LoginButton => LoginFocus::Password,
            LoginFocus::CreateUserButton => LoginFocus::LoginButton,
        };
    }

    fn nav_up(&mut self) {
        if self.focus == LoginFocus::User && self.user_dropdown_open {
            if self.user_selected > 0 {
                self.user_selected -= 1;
            }
        } else {
            self.focus_prev();
        }
    }

    fn nav_down(&mut self) {
        if self.focus == LoginFocus::User && self.user_dropdown_open {
            if self.user_selected + 1 < self.user_options.len() {
                self.user_selected += 1;
            }
        } else {
            self.focus_next();
        }
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
            _ => ScreenResult::None,
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
        for (idx, name) in self.user_options.iter().take(max_items).enumerate() {
            let style = if idx == self.user_selected {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default()
            };
            lines.push(Line::from(Span::styled(format!("> {}", name), style)));
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
        let form_height = 7 + dropdown_height;
        let form_area = centered_rect(area, 52, form_height);
        let form_block = Block::default().title("Login").borders(Borders::ALL);
        let inner = form_block.inner(form_area);
        frame.render_widget(form_block, form_area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(dropdown_height),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        let label_style = Style::default().add_modifier(Modifier::BOLD);
        let field_style = Style::default().fg(Color::White);
        let focus_style = Style::default().fg(Color::Black).bg(Color::White);
        let field_focus_style = Style::default().fg(Color::Black).bg(Color::White);

        render_field(
            frame,
            rows[0],
            "User",
            Line::from(vec![Span::styled(
                format!(
                    "[ {} {} ]",
                    self.selected_user_label(),
                    if self.user_dropdown_open { "^" } else { "v" }
                ),
                if self.focus == LoginFocus::User {
                    field_focus_style
                } else {
                    field_style
                },
            )]),
            label_style,
            self.focus == LoginFocus::User,
            focus_style,
        );
        if self.user_dropdown_open {
            let options = self.dropdown_lines();
            let list = Paragraph::new(options);
            frame.render_widget(list, rows[1]);
        }
        render_field(
            frame,
            rows[2],
            "Password",
            Line::from(vec![Span::styled(
                format!("[ {} ]", "*".repeat(self.password_input.len())),
                if self.focus == LoginFocus::Password {
                    field_focus_style
                } else {
                    field_style
                },
            )]),
            label_style,
            self.focus == LoginFocus::Password,
            focus_style,
        );

        let normal = Style::default();
        let login_style = match self.focus {
            LoginFocus::LoginButton => focus_style,
            _ => normal,
        };
        let create_style = match self.focus {
            LoginFocus::CreateUserButton => focus_style,
            _ => normal,
        };

        let buttons = Paragraph::new(Line::from(vec![
            Span::styled(" Login ", login_style),
            Span::raw("  "),
            Span::styled(" Create new user ", create_style),
        ]));
        frame.render_widget(buttons, rows[4]);
    }

    fn handle_action(&mut self, action: Action, _repo: &mut dyn EntryRepository) -> ScreenResult {
        match action {
            Action::Quit => ScreenResult::Quit,
            Action::FocusNext | Action::NavRight => {
                self.focus_next();
                ScreenResult::None
            }
            Action::FocusPrev | Action::NavLeft => {
                self.focus_prev();
                ScreenResult::None
            }
            Action::NavUp => {
                self.nav_up();
                ScreenResult::None
            }
            Action::NavDown => {
                self.nav_down();
                ScreenResult::None
            }
            Action::Activate => self.activate(),
            Action::InputChar(ch) => {
                if self.focus == LoginFocus::Password {
                    self.password_input.push(ch);
                }
                // Handle user selection logic if typable? Not implemented yet.
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

fn render_field(
    frame: &mut ratatui::Frame<'_>,
    area: ratatui::layout::Rect,
    label: &str,
    value: Line<'_>,
    label_style: Style,
    focused: bool,
    focus_style: Style,
) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(area);

    let label = Paragraph::new(Line::from(Span::styled(
        label,
        if focused { focus_style } else { label_style },
    )));
    frame.render_widget(label, cols[0]);

    let value = Paragraph::new(value);
    frame.render_widget(value, cols[1]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoginFocus {
    User,
    Password,
    LoginButton,
    CreateUserButton,
}
