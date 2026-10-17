use crate::app::events::AppEvent;
use crate::chars::Icon;
use crate::components::Component;
use crate::config::VERSION;
use crate::prelude::*;

pub struct LogosFooter {
    app_name: String,
    version: String,
    help: String,
    keymaps: String,
}

impl LogosFooter {
    pub fn new() -> Self {
        Self {
            app_name: format!(" {} logos ", Icon::App.as_str()),
            version: format!("[{VERSION}]"),
            help: String::from("?:help  "),
            keymaps: String::from("q:quit "),
        }
    }
}

impl Component for LogosFooter {
    fn update(&mut self, _event: &AppEvent) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let [left_area, help_area, right_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(self.help.len() as u16),
            Constraint::Length(self.keymaps.len() as u16),
        ])
        .areas(area);

        Line::from(vec![
            Span::styled(&self.app_name, theme::FOOTER_APP_NAME),
            Span::styled(&self.version, theme::FOOTER_VERSION),
        ])
        .style(theme::FOOTER)
        .render(left_area, buf);

        Line::from(self.help.as_str())
            .style(theme::FOOTER.patch(theme::FOOTER_HELP))
            .render(help_area, buf);

        Line::from(self.keymaps.as_str())
            .style(theme::FOOTER.patch(theme::FOOTER_KEYMAPS))
            .render(right_area, buf);

        Ok(())
    }
}
