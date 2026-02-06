use domain::{DomainError, Entry, EntryFilter, EntryRepository};
use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::{Screen, ScreenResult};
use crate::event::Action;
use crate::layout::main_chunks;

pub struct DashboardScreen {
    entries: Vec<Entry>,
    list_state: ListState,
}

impl DashboardScreen {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            list_state: ListState::default(),
        }
    }

    fn refresh_entries(&mut self, repo: &dyn EntryRepository) -> Result<(), DomainError> {
        // TODO: Pagination? For now list all.
        self.entries = repo.list(EntryFilter::default())?;
        if self.entries.is_empty() {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
        Ok(())
    }
}

impl Screen for DashboardScreen {
    fn init(&mut self, repo: &mut dyn EntryRepository) -> Result<(), DomainError> {
        self.refresh_entries(repo)
    }

    fn render(&mut self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.size();
        let chunks = main_chunks(area);

        let header = Block::default().title("TUI Money").borders(Borders::ALL);
        frame.render_widget(header, chunks[0]);

        // Dashboard Content
        if self.entries.is_empty() {
            let body = Paragraph::new("No entries found. Press 'r' to reload.")
                .block(Block::default().title("Dashboard").borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(body, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .entries
                .iter()
                .map(|entry| {
                    let amount_style = if entry.amount.is_negative() {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Green)
                    };

                    let content = Line::from(vec![
                        Span::styled(
                            format!("{:<12}", entry.occurred_on.format("%Y-%m-%d")),
                            Style::default(),
                        ),
                        Span::raw(" "),
                        Span::styled(
                            format!("{:<15}", entry.category.as_str()),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                        Span::styled(format!("{}", entry.amount), amount_style),
                    ]);
                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Entries").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol(">> ");

            frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
        }

        let footer =
            Paragraph::new("[q] quit  [r] reload").block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[2]);
    }

    fn handle_action(&mut self, action: Action, repo: &mut dyn EntryRepository) -> ScreenResult {
        match action {
            Action::Quit => ScreenResult::Quit,
            Action::InputChar('r') => {
                let _ = self.refresh_entries(repo);
                ScreenResult::None
            }
            Action::NavDown | Action::FocusNext => {
                if !self.entries.is_empty() {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i >= self.entries.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                }
                ScreenResult::None
            }
            Action::NavUp | Action::FocusPrev => {
                if !self.entries.is_empty() {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.entries.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                }
                ScreenResult::None
            }
            _ => ScreenResult::None,
        }
    }
}
