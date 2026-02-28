use crate::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, BorderType, Borders};

pub struct BooksView<'a> {
    pub books: &'a Vec<String>,
    pub selected_book_index: usize,
    pub scrolled_offset: usize,
}

impl<'a> Widget for BooksView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Books ".yellow().bold());

        let inner = block.inner(area);
        block.render(area, buf);

        for (row, (i, b)) in self
            .books
            .iter()
            .enumerate()
            .skip(self.scrolled_offset)
            .take(inner.height as usize)
            .enumerate()
        {
            let line_area = Rect {
                y: inner.y + row as u16,
                height: 1,
                ..inner
            };
            if i == self.selected_book_index {
                Line::from(format!("> {}", b))
                    .cyan()
                    .bold()
                    .render(line_area, buf);
            } else {
                Line::from(format!("  {}", b)).render(line_area, buf);
            }
        }
    }
}
