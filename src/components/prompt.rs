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
    /// Shown in place of the input after a failed command, until the next key.
    error: Option<String>,
    /// Shown dimmed after the input, accepted with `Tab`.
    suggestion: Option<String>,
}

impl Prompt {
    pub fn new() -> Self {
        Prompt {
            input: String::new(),
            error: None,
            suggestion: None,
        }
    }

    /// The input is kept, so the next key goes back to editing it.
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn set_suggestion(&mut self, suggestion: Option<String>) {
        self.suggestion = suggestion;
    }

    pub fn key(&mut self, key: KeyEvent) -> PromptKey {
        // Esc only leaves the error.  Any other key leaves it and still applies to the input.
        if self.error.take().is_some() && key.code == KeyCode::Esc {
            return PromptKey::Edit;
        }
        match key.code {
            KeyCode::Esc => PromptKey::Cancel,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => PromptKey::Cancel,
            KeyCode::Enter => PromptKey::Submit(self.input.clone()),
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
            KeyCode::Tab => {
                if let Some(suggestion) = self.suggestion.take() {
                    self.input.push_str(&suggestion);
                }
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
                // The cursor sits on the first suggested character, like fish.
                let suggestion = self.suggestion.as_deref().unwrap_or("");
                let mut rest = suggestion.chars();
                let under_cursor = rest.next().map_or(" ".to_string(), String::from);
                Line::from(vec![
                    Span::styled(":", theme::PROMPT),
                    Span::styled(input, theme::PROMPT),
                    Span::styled(under_cursor, theme::PROMPT_CURSOR),
                    Span::styled(rest.as_str(), theme::PROMPT_SUGGESTION),
                ])
            }
        };
        // Covers whatever is underneath, e.g. the footer.
        Clear.render(area, buf);
        line.style(theme::PROMPT).render(area, buf);
        Ok(())
    }
}
