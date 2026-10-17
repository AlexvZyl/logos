use crate::app::events::{AppEvent, KeyMap};
use crate::components::Component;
use crate::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding};
use unicode_width::UnicodeWidthStr;

// `(left, right, top, bottom)`.
const PADDING: Padding = Padding::new(4, 4, 1, 1);
const COLUMN_GAP: &str = "   ";
const BORDERS: u16 = 2;

pub struct HelpOverlay {
    entries: Vec<(String, &'static str)>,
}

impl HelpOverlay {
    pub fn new(keymap: &KeyMap) -> Self {
        Self {
            entries: keymap.help_entries(),
        }
    }
}

impl Component for HelpOverlay {
    fn update(&mut self, _event: &AppEvent) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let key_width = self.entries.iter().map(|(k, _)| k.width()).max().unwrap_or(0);
        let desc_width = self.entries.iter().map(|(_, d)| d.width()).max().unwrap_or(0);

        let content_width = (key_width + COLUMN_GAP.len() + desc_width) as u16;
        let width = (content_width + PADDING.left + PADDING.right + BORDERS).min(area.width);
        let height =
            (self.entries.len() as u16 + PADDING.top + PADDING.bottom + BORDERS).min(area.height);
        let popup = Rect {
            x: area.x + (area.width - width) / 2,
            y: area.y + (area.height - height) / 2,
            width,
            height,
        };

        let lines: Vec<Line> = self
            .entries
            .iter()
            .map(|(keys, description)| {
                Line::from(vec![
                    Span::styled(format!("{keys:<key_width$}"), theme::KEY_HINT),
                    Span::raw(COLUMN_GAP),
                    Span::raw(*description),
                ])
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::BORDER_FOCUSED)
            .padding(PADDING)
            .title(Span::styled(" Keybinds ", theme::TITLE));

        Clear.render(popup, buf);
        Paragraph::new(lines).block(block).render(popup, buf);
        Ok(())
    }
}
