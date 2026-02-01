use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::layout::main_chunks;

pub struct DashboardScreen {
    hint: String,
}

impl DashboardScreen {
    pub fn new() -> Self {
        Self {
            hint: "Press 'q' to exit.".to_string(),
        }
    }

    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.size();
        let chunks = main_chunks(area);

        let header = Block::default().title("TUI Money").borders(Borders::ALL);
        frame.render_widget(header, chunks[0]);

        let body = Paragraph::new(self.hint.as_str())
            .block(Block::default().title("Dashboard").borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(body, chunks[1]);

        let footer = Paragraph::new("[q] quit  [d] dashboard")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[2]);
    }
}
