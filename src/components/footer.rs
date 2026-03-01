use crate::app::events::AppEvent;
use crate::components::Component;
use crate::config::VERSION;
use crate::prelude::*;

pub struct LogosFooter {
    left_side: String,
    right_side: String,
}

impl LogosFooter {
    pub fn new() -> Self {
        Self {
            // TODO: Check for these icons support before just rendering it.
            left_side: format!("   logos [{}]", VERSION),
            right_side: String::from("q: quit "),
        }
    }
}

impl Component for LogosFooter {
    fn update(&mut self, _event: &AppEvent) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let left = Line::from(self.left_side.as_str());
        let right = Line::from(self.right_side.as_str()).right_aligned();

        let [left_area, right_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(self.right_side.len() as u16),
        ])
        .areas(area);

        Paragraph::new(left)
            .bg(Color::Black)
            .bold()
            .white()
            .render(left_area, buf);
        Paragraph::new(right)
            .bg(Color::Black)
            .bold()
            .red()
            .render(right_area, buf);
        Ok(())
    }
}
