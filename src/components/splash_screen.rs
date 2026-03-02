use crate::app::events::AppEvent;
use crate::components::Component;
use crate::config::VERSION;
use crate::prelude::*;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::Stylize;
use ratatui::text::{Line, Text};
use tui_big_text::{BigText, PixelSize};

// TODO: Revisit and rename components here.
// Should probably move menu out.
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
            .italic()
            .dark_gray();

        let menu = Paragraph::new(Text::from(vec![
            Line::from(vec!["[r]".cyan().bold(), " Reader".white()]),
            Line::from(vec!["[q]".red().bold(), " Quit  ".white()]),
        ]))
        .alignment(Alignment::Center);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Fill(1),
            ])
            .split(rect);

        big_text.render(chunks[1], buf);
        version.render(chunks[2], buf);
        menu.render(chunks[4], buf);
        Ok(())
    }
}
