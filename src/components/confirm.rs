use crate::app::events::AppEvent;
use crate::components::Component;
use crate::prelude::*;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding};
use unicode_width::UnicodeWidthStr;

// `(left, right, top, bottom)`.
const PADDING: Padding = Padding::new(4, 4, 1, 1);
const BORDERS: u16 = 2;

/// A centred yes/no popup.
pub struct Confirm {
    message: &'static str,
}

impl Confirm {
    pub fn new(message: &'static str) -> Self {
        Confirm { message }
    }

    /// `Some(answer)` once answered.  Other keys are ignored, so a stray key does not decide.
    pub fn key(&self, key: KeyEvent) -> Option<bool> {
        match key.code {
            KeyCode::Char('y' | 'Y') | KeyCode::Enter => Some(true),
            KeyCode::Char('n' | 'N') | KeyCode::Esc => Some(false),
            _ => None,
        }
    }
}

impl Component for Confirm {
    fn update(&mut self, _event: &AppEvent) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let answers = Line::from(vec![
            Span::styled("[y]", theme::KEY_HINT_DANGER),
            Span::styled(" Yes   ", theme::TEXT),
            Span::styled("[n]", theme::KEY_HINT),
            Span::styled(" No", theme::TEXT),
        ]);
        let content_width = self.message.width().max(answers.width()) as u16;
        let text = Text::from(vec![
            Line::styled(self.message, theme::TEXT),
            Line::default(),
            answers,
        ]);

        let width = (content_width + PADDING.left + PADDING.right + BORDERS).min(area.width);
        let height =
            (text.height() as u16 + PADDING.top + PADDING.bottom + BORDERS).min(area.height);
        let popup = Rect {
            x: area.x + (area.width - width) / 2,
            y: area.y + (area.height - height) / 2,
            width,
            height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::BORDER_FOCUSED)
            .padding(PADDING);

        Clear.render(popup, buf);
        Paragraph::new(text).centered().block(block).render(popup, buf);
        Ok(())
    }
}
