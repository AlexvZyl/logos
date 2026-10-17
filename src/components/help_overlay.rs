use crate::app::events::{AppEvent, KeyMap};
use crate::chars::Symbol;
use crate::components::Component;
use crate::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding};
use unicode_width::UnicodeWidthStr;

// `(left, right, top, bottom)`.
const PADDING: Padding = Padding::new(4, 4, 1, 1);
const COLUMN_GAP: &str = "   ";
const BORDERS: u16 = 2;

pub struct HelpOverlay {
    sections: Vec<Vec<(String, &'static str)>>,
}

impl HelpOverlay {
    pub fn new(keymap: &KeyMap) -> Self {
        Self {
            sections: keymap.help_entries(),
        }
    }
}

impl Component for HelpOverlay {
    fn update(&mut self, _event: &AppEvent) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let entries = || self.sections.iter().flatten();
        let key_width = entries().map(|(k, _)| k.width()).max().unwrap_or(0);
        let desc_width = entries().map(|(_, d)| d.width()).max().unwrap_or(0);

        let content_width = key_width + COLUMN_GAP.len() + desc_width;

        let mut lines: Vec<Line> = Vec::new();
        for (i, entries) in self.sections.iter().enumerate() {
            if i > 0 {
                let line = Symbol::HorizontalLine.as_str().repeat(content_width);
                lines.push(Line::styled(line, theme::MUTED));
            }
            lines.extend(entries.iter().map(|(keys, description)| {
                Line::from(vec![
                    Span::styled(format!("{keys:<key_width$}"), theme::KEY_HINT),
                    Span::raw(COLUMN_GAP),
                    Span::raw(*description),
                ])
            }));
        }

        let width = (content_width as u16 + PADDING.left + PADDING.right + BORDERS).min(area.width);
        let height =
            (lines.len() as u16 + PADDING.top + PADDING.bottom + BORDERS).min(area.height);
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
            .padding(PADDING)
            .title(Span::styled(" Help ", theme::TITLE));

        Clear.render(popup, buf);
        Paragraph::new(lines).block(block).render(popup, buf);
        Ok(())
    }
}
