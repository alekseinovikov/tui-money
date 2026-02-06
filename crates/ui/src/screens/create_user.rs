use domain::EntryRepository;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::{Screen, ScreenId, ScreenResult};
use crate::event::Action;
use crate::layout::centered_rect;

pub struct CreateUserScreen {
    focus: CreateUserFocus,
    login_input: String,
    password_input: String,
    repeat_input: String,
}

impl CreateUserScreen {
    pub fn new() -> Self {
        Self {
            focus: CreateUserFocus::Login,
            login_input: String::new(),
            password_input: String::new(),
            repeat_input: String::new(),
        }
    }

    fn focus_next(&mut self) {
        self.focus = match self.focus {
            CreateUserFocus::Login => CreateUserFocus::Password,
            CreateUserFocus::Password => CreateUserFocus::RepeatPassword,
            CreateUserFocus::RepeatPassword => CreateUserFocus::CreateButton,
            CreateUserFocus::CreateButton => CreateUserFocus::BackButton,
            CreateUserFocus::BackButton => CreateUserFocus::Login,
        };
    }

    fn focus_prev(&mut self) {
        self.focus = match self.focus {
            CreateUserFocus::Login => CreateUserFocus::BackButton,
            CreateUserFocus::Password => CreateUserFocus::Login,
            CreateUserFocus::RepeatPassword => CreateUserFocus::Password,
            CreateUserFocus::CreateButton => CreateUserFocus::RepeatPassword,
            CreateUserFocus::BackButton => CreateUserFocus::CreateButton,
        };
    }

    fn activate(&self) -> ScreenResult {
        match self.focus {
            CreateUserFocus::CreateButton => {
                // TODO: Create user logic.
                ScreenResult::Go(ScreenId::Login)
            }
            CreateUserFocus::BackButton => ScreenResult::Go(ScreenId::Login),
            _ => ScreenResult::None,
        }
    }

    fn input_char(&mut self, ch: char) {
        match self.focus {
            CreateUserFocus::Login => self.login_input.push(ch),
            CreateUserFocus::Password => self.password_input.push(ch),
            CreateUserFocus::RepeatPassword => self.repeat_input.push(ch),
            _ => {}
        }
    }

    fn backspace(&mut self) {
        match self.focus {
            CreateUserFocus::Login => self.login_input.pop(),
            CreateUserFocus::Password => self.password_input.pop(),
            CreateUserFocus::RepeatPassword => self.repeat_input.pop(),
            _ => None,
        };
    }

    fn nav_up(&mut self) {
        self.focus_prev();
    }

    fn nav_down(&mut self) {
        self.focus_next();
    }
}

impl Screen for CreateUserScreen {
    fn render(&mut self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.size();
        frame.render_widget(Clear, area);

        let form_area = centered_rect(area, 58, 11);
        let form_block = Block::default()
            .title("Create New User")
            .borders(Borders::ALL);
        let inner = form_block.inner(form_area);
        frame.render_widget(form_block, form_area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
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
            "Login",
            Line::from(vec![Span::styled(
                format!("[ {} ]", self.login_input),
                if self.focus == CreateUserFocus::Login {
                    field_focus_style
                } else {
                    field_style
                },
            )]),
            label_style,
            self.focus == CreateUserFocus::Login,
            focus_style,
        );
        render_field(
            frame,
            rows[1],
            "Password",
            Line::from(vec![Span::styled(
                format!("[ {} ]", "*".repeat(self.password_input.len())),
                if self.focus == CreateUserFocus::Password {
                    field_focus_style
                } else {
                    field_style
                },
            )]),
            label_style,
            self.focus == CreateUserFocus::Password,
            focus_style,
        );
        render_field(
            frame,
            rows[2],
            "Repeat",
            Line::from(vec![Span::styled(
                format!("[ {} ]", "*".repeat(self.repeat_input.len())),
                if self.focus == CreateUserFocus::RepeatPassword {
                    field_focus_style
                } else {
                    field_style
                },
            )]),
            label_style,
            self.focus == CreateUserFocus::RepeatPassword,
            focus_style,
        );

        let normal = Style::default();
        let create_style = match self.focus {
            CreateUserFocus::CreateButton => focus_style,
            _ => normal,
        };
        let back_style = match self.focus {
            CreateUserFocus::BackButton => focus_style,
            _ => normal,
        };

        let buttons = Paragraph::new(Line::from(vec![
            Span::styled(" Create ", create_style),
            Span::raw("  "),
            Span::styled(" Back ", back_style),
        ]));
        frame.render_widget(buttons, rows[4]);
    }

    fn handle_action(&mut self, action: Action, _repo: &mut dyn EntryRepository) -> ScreenResult {
        match action {
            Action::Quit => ScreenResult::Quit,
            Action::FocusNext | Action::NavRight | Action::NavDown => {
                self.nav_down();
                ScreenResult::None
            }
            Action::FocusPrev | Action::NavLeft | Action::NavUp => {
                self.nav_up();
                ScreenResult::None
            }
            Action::Activate => self.activate(),
            Action::InputChar(ch) => {
                self.input_char(ch);
                ScreenResult::None
            }
            Action::Backspace => {
                self.backspace();
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
enum CreateUserFocus {
    Login,
    Password,
    RepeatPassword,
    CreateButton,
    BackButton,
}
