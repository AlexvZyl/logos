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
    fn update(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Focus => self.focused = true,
            AppEvent::Defocus => self.focused = false,
            _ => {}
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Strong's ".yellow().bold())
            .border_style(if self.focused {
                Style::default().blue()
            } else {
                Style::default()
            });

        block.render(area, buf);
    }
}
