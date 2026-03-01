use crate::app::events::AppEvent;
use crate::components::Component;
use crate::config::VERSION;
use crate::prelude::*;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::Stylize;
use tui_big_text::{BigText, PixelSize};

pub struct SplashScreen;

impl Component for SplashScreen {
    fn update(&mut self, _event: &AppEvent) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, rect: Rect, buf: &mut Buffer) -> Result<()> {
        let big_text = BigText::builder()
            .pixel_size(PixelSize::Full)
            .centered()
            .lines(vec!["LOGOS".yellow().into()])
            .build();

        let version = Paragraph::new(format!("v{VERSION}"))
            .alignment(Alignment::Center)
            .bold()
            .dark_gray();

        let prompt = Paragraph::new(format!("Press any key to enter"))
            .italic()
            .alignment(Alignment::Center);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(rect);

        big_text.render(chunks[1], buf);
        version.render(chunks[2], buf);
        prompt.render(chunks[4], buf);
        Ok(())
    }
}
