use crate::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem};

pub struct BooksView<'a> {
    pub books: &'a Vec<String>,
}

impl<'a> Widget for BooksView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .books
            .iter()
            .map(|b| ListItem::new(b.as_str()))
            .collect();

        List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Books ".yellow()),
            )
            .render(area, buf);
    }
}
