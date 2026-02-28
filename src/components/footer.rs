use ratatui::style::Stylize;

use crate::app::events::AppEvent;
use crate::components::Component;
use crate::config::VERSION;
use crate::prelude::*;

pub struct LogosFooter;

impl Component for LogosFooter {
    fn update(&mut self, _event: &AppEvent) {}

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(vec![
            Span::raw(format!(" logos [{}]", VERSION)),
            Span::raw(format!(
                "{:>width$}",
                "q: quit ",
                width = area.width as usize - 14
            ))
            .bold()
            .red(),
        ]);
        Paragraph::new(line)
            .style(Style::default().bg(Color::Black))
            .bold()
            .render(area, buf);
    }
}
