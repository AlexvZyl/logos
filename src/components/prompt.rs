use crate::app::events::AppEvent;
use crate::components::Component;
use crate::prelude::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::Clear;
use unicode_width::UnicodeWidthStr;

/// What the prompt wants done after a key.
pub enum PromptKey {
    /// Still typing.
    Edit,
    Cancel,
    /// The input, without the leading `:`.
    Submit(String),
}

/// The `:` command line, drawn over the bottom row.
pub struct Prompt {
    input: String,
    /// Shown in place of the input after a failed command.  Any key then closes the prompt.
    error: Option<String>,
}

impl Prompt {
    pub fn new() -> Self {
        Prompt {
            input: String::new(),
            error: None,
        }
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn key(&mut self, key: KeyEvent) -> PromptKey {
        assert!(self.error.is_none());
        match key.code {
            KeyCode::Esc => PromptKey::Cancel,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => PromptKey::Cancel,
            KeyCode::Enter => PromptKey::Submit(std::mem::take(&mut self.input)),
            // Like vim, deleting past the start closes the prompt.
            KeyCode::Backspace if self.input.is_empty() => PromptKey::Cancel,
            KeyCode::Backspace => {
                self.input.pop();
                PromptKey::Edit
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                PromptKey::Edit
            }
            _ => PromptKey::Edit,
        }
    }

    /// The bottom row of `area`.
    pub fn area(area: Rect) -> Rect {
        Rect {
            y: area.bottom().saturating_sub(1),
            height: area.height.min(1),
            ..area
        }
    }
}

impl Component for Prompt {
    fn update(&mut self, _event: &AppEvent) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let line = match &self.error {
            Some(error) => Line::styled(error.as_str(), theme::PROMPT_ERROR),
            None => {
                // Keep the end of long input in view, with room for `:` and the cursor.
                let room = (area.width as usize).saturating_sub(2);
                let mut input = self.input.as_str();
                while input.width() > room {
                    let mut chars = input.chars();
                    chars.next();
                    input = chars.as_str();
                }
                Line::from(vec![
                    Span::styled(":", theme::PROMPT),
                    Span::styled(input, theme::PROMPT),
                    Span::styled(" ", theme::PROMPT_CURSOR),
                ])
            }
        };
        // Covers whatever is underneath, e.g. the footer.
        Clear.render(area, buf);
        line.style(theme::PROMPT).render(area, buf);
        Ok(())
    }
}
