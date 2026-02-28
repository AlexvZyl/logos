use crate::prelude::*;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::Stylize;
use tui_big_text::{BigText, PixelSize};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct SplashScreen;

impl Widget for SplashScreen {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let big_text = BigText::builder()
            .pixel_size(PixelSize::Full)
            .centered()
            .lines(vec!["LOGOS".yellow().into()])
            .build();

        let version = Paragraph::new(format!("v{VERSION}"))
            .alignment(Alignment::Center)
            .bold().italic();


        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(rect);

        big_text.render(chunks[1], buf);
        version.render(chunks[2], buf);
    }
}
