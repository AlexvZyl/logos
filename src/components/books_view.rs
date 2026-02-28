use crate::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem};

pub struct BooksView<'a> {
    pub books: &'a Vec<String>,
    pub selected_book_index: usize,
    pub scrolled_offset: usize,
}

impl<'a> Widget for BooksView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let visible = area.height.saturating_sub(2) as usize;

        let items: Vec<ListItem> = self
            .books
            .iter()
            .enumerate()
            .skip(self.scrolled_offset)
            .take(visible)
            .map(|(i, b)| {
                let content = format!("> {}", b);
                if i == self.selected_book_index {
                    ListItem::new(content).cyan().bold()
                } else {
                    ListItem::new(format!("  {}", b))
                }
            })
            .collect(); // TODO: Don't want this collect.

        List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Books ".yellow().bold()),
            )
            .render(area, buf);
    }
}
