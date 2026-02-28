use crate::config::VERSION;
use crate::prelude::*;

pub struct LogosFooter;

impl Widget for LogosFooter {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(vec![
            Span::raw(format!(" logos [{}]", VERSION)),
            Span::raw(format!(
                "{:>width$}",
                "q: quit ",
                width = area.width as usize - 14
            )),
        ]);
        Paragraph::new(line)
            .style(Style::default().bg(Color::Black))
            .render(area, buf);
    }
}
