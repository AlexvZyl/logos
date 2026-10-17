use crate::app::events::AppEvent;
use crate::components::Component;
use crate::config::VERSION;
use crate::prelude::*;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
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
            .lines(vec![Line::styled("LOGOS", theme::LOGO)])
            .build();

        let version = Paragraph::new(VERSION)
            .alignment(Alignment::Center)
            .style(theme::MUTED.italic());

        let menu = Paragraph::new(Text::from(vec![
            Line::from(vec![Span::styled("[r]", theme::KEY_HINT), Span::styled(" Reader", theme::TEXT)]),
            Line::from(vec![Span::styled("[?]", theme::KEY_HINT), Span::styled(" Help  ", theme::TEXT)]),
            Line::from(vec![Span::styled("[q]", theme::KEY_HINT_DANGER), Span::styled(" Quit  ", theme::TEXT)]),
        ]))
        .alignment(Alignment::Center);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .split(rect);

        big_text.render(chunks[1], buf);
        version.render(chunks[2], buf);
        menu.render(chunks[4], buf);
        Ok(())
    }
}
