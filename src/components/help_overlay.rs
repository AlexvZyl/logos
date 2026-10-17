use crate::app::events::{AppEvent, KeyMap};
use crate::components::Component;
use crate::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding};
use unicode_width::UnicodeWidthStr;

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

        // Borders (2) + padding (4) + gap between columns (3).
        let width = ((key_width + desc_width + 9) as u16).min(area.width);
        // Borders (2) + padding (2).
        let height = ((self.entries.len() + 4) as u16).min(area.height);
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
                    Span::styled(format!("{keys:<key_width$}"), Style::default().yellow()),
                    Span::raw("   "),
                    Span::raw(*description),
                ])
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().blue())
            .padding(Padding::new(2, 2, 1, 1))
            .title(" Keybinds ".yellow().bold());

        Clear.render(popup, buf);
        Paragraph::new(lines).block(block).render(popup, buf);
        Ok(())
    }
}
