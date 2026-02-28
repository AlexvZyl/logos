use crate::app::events::{AppEvent, UserAction};
use crate::components::Component;
use crate::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, BorderType, Borders};

pub struct BooksView {
    books: Vec<String>,
    selected_book_index: usize,
    scrolled_offset: usize,
    focused: bool,
}

impl BooksView {
    pub fn new(books: Vec<String>) -> Self {
        BooksView {
            books,
            selected_book_index: 0,
            scrolled_offset: 0,
            focused: false,
        }
    }

    pub fn selected_book(&self) -> &str {
        &self.books[self.selected_book_index]
    }

    pub fn focused(&self) -> bool {
        self.focused
    }
}

impl Component for BooksView {
    fn update(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Focus => self.focused = true,
            AppEvent::Defocus => self.focused = false,
            AppEvent::UserAction(action) if self.focused => match action {
                UserAction::MoveDown => {
                    if self.selected_book_index < self.books.len() - 1 {
                        self.selected_book_index += 1;
                    }
                }
                UserAction::MoveUp => {
                    if self.selected_book_index > 0 {
                        self.selected_book_index -= 1;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Books ".yellow().bold())
            .border_style(if self.focused {
                Style::default().blue()
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        block.render(area, buf);

        let visible = inner.height as usize;
        let min_index = self.scrolled_offset;
        let max_index = self.scrolled_offset + visible - 1;
        if self.selected_book_index < min_index {
            self.scrolled_offset -= min_index - self.selected_book_index;
        }
        if self.selected_book_index > max_index {
            self.scrolled_offset += self.selected_book_index - max_index;
        }

        for (row, (i, b)) in self
            .books
            .iter()
            .enumerate()
            .skip(self.scrolled_offset)
            .take(visible)
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
