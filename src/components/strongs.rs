use crate::app::events::AppEvent;
use crate::components::Component;
use crate::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders};

pub struct Strongs {
    focused: bool,
}

impl Strongs {
    pub fn new() -> Self {
        Self { focused: false }
    }
}

impl Component for Strongs {
    fn update(&mut self, event: &AppEvent) -> Result<()> {
        match event {
            AppEvent::Focus => self.focused = true,
            AppEvent::Defocus => self.focused = false,
            _ => {}
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" [4] Strong's ", theme::TITLE))
            .border_style(theme::border(self.focused));

        block.render(area, buf);
        Ok(())
    }
}
