use crate::app::events::AppEvent;
use crate::components::Component;
use crate::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders};

pub struct References {
    focused: bool,
}

impl References {
    pub fn new() -> Self {
        Self { focused: false }
    }
}

impl Component for References {
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
            .title(" [3] References ".yellow().bold())
            .border_style(if self.focused {
                Style::default().blue()
            } else {
                Style::default()
            });

        block.render(area, buf);
        Ok(())
    }
}
