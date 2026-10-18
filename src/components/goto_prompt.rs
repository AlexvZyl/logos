use crate::app::events::AppEvent;
use crate::components::Component;
use crate::components::book_column::Position;
use crate::prelude::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::{Block, BorderType, Borders, Clear};

const WIDTH: u16 = 28;
const BORDERS: u16 = 2;

/// Result of feeding a key to the prompt.
pub enum PromptOutcome {
    /// Still editing.
    Pending,
    Cancel,
    Submit(Position),
}

/// Small popup for typing a `chapter[:verse]` reference.
pub struct GotoPrompt {
    input: String,
    error: Option<String>,
}

impl GotoPrompt {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            error: None,
        }
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> PromptOutcome {
        match key.code {
            KeyCode::Esc => return PromptOutcome::Cancel,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return PromptOutcome::Cancel;
            }
            KeyCode::Enter => match parse(&self.input) {
                Some(position) => return PromptOutcome::Submit(position),
                None => self.error = Some("Expected chapter[:verse]".to_string()),
            },
            KeyCode::Backspace => {
                self.input.pop();
                self.error = None;
            }
            KeyCode::Char(c @ ('0'..='9' | ':')) => {
                self.input.push(c);
                self.error = None;
            }
            _ => {}
        }
        PromptOutcome::Pending
    }
}

/// `"3"` is chapter 3, `"3:2"` is chapter 3 verse 2.  Verse `0` is the chapter heading.
fn parse(input: &str) -> Option<Position> {
    let (chapter, verse) = match input.split_once(':') {
        Some((chapter, verse)) => (chapter, Some(verse)),
        None => (input, None),
    };
    let chapter = chapter.parse().ok().filter(|&c| c > 0)?;
    let verse = match verse {
        // A trailing colon, e.g. `"3:"`, is still just the chapter.
        None | Some("") => 0,
        Some(verse) => verse.parse().ok().filter(|&v| v > 0)?,
    };
    Some(Position { chapter, verse })
}

impl Component for GotoPrompt {
    fn update(&mut self, _event: &AppEvent) -> Result<()> {
        Ok(())
    }

    /// Centred in `area`.
    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let mut lines = vec![Line::from(vec![
            Span::styled("> ", theme::KEY_HINT),
            Span::raw(self.input.as_str()),
            Span::styled("█", theme::MUTED),
        ])];
        if let Some(error) = &self.error {
            lines.push(Line::styled(error.as_str(), theme::KEY_HINT_DANGER));
        }

        let width = WIDTH.min(area.width);
        let height = (lines.len() as u16 + BORDERS).min(area.height);
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
            .title(Span::styled(" Go to chapter:verse ", theme::TITLE));

        Clear.render(popup, buf);
        Paragraph::new(lines).block(block).render(popup, buf);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(chapter: usize, verse: usize) -> Option<Position> {
        Some(Position { chapter, verse })
    }

    #[test]
    fn parses_references() {
        assert_eq!(parse("3"), pos(3, 0));
        assert_eq!(parse("3:"), pos(3, 0));
        assert_eq!(parse("3:2"), pos(3, 2));
        assert_eq!(parse("119:176"), pos(119, 176));
    }

    #[test]
    fn rejects_malformed_references() {
        for input in ["", ":", ":2", "0", "3:0", "3:2:1", "3::2"] {
            assert_eq!(parse(input), None, "{input:?}");
        }
    }
}
