use crate::app::events::AppEvent;
use crate::components::Component;
use crate::config::VERSION;
use crate::prelude::*;

pub struct LogosFooter {
    app_name: String,
    version: String,
    keymaps: String,
}

impl LogosFooter {
    pub fn new() -> Self {
        Self {
            // TODO: Check for these icons support before just rendering it.
            app_name: "   logos ".to_string(),
            version: format!("[{}]", VERSION.to_string()),
            keymaps: String::from("[q] quit "),
        }
    }
}

impl Component for LogosFooter {
    fn update(&mut self, _event: &AppEvent) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let [left_area, right_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(self.keymaps.len() as u16),
        ])
        .areas(area);

        Line::from(vec![
            Span::styled(&self.app_name, Style::new().white().bold()),
            Span::styled(&self.version, Style::new().dark_gray().bold()),
        ])
        .bg(Color::Black)
        .render(left_area, buf);

        Line::from(self.keymaps.as_str())
            .bg(Color::Black)
            .bold()
            .red()
            .render(right_area, buf);

        Ok(())
    }
}
