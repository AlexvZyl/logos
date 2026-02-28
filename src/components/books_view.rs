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

    // Pre-built lines to reduce cost of rendering.
    all_lines: Vec<Line<'static>>,
    selected_lines: Vec<Line<'static>>,
}

impl BooksView {
    pub fn new(books: Vec<String>) -> Self {
        let all_lines = BooksView::build_all_lines(&books, "  ");
        let selected_lines = BooksView::build_all_lines(&books, "> ")
            .into_iter()
            .map(|l| l.cyan().bold())
            .collect();
        BooksView {
            books,
            selected_book_index: 0,
            scrolled_offset: 0,
            focused: false,
            all_lines,
            selected_lines,
        }
    }

    pub fn selected_book(&self) -> &str {
        &self.books[self.selected_book_index]
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    fn build_all_lines(books: &Vec<String>, prefix: &str) -> Vec<Line<'static>> {
        books
            .iter()
            .map(|b| Line::from(format!("{}{}", prefix, b)))
            .collect()
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

        for (row, (i, _)) in self
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
            let line = if i == self.selected_book_index {
                &self.selected_lines[i]
            } else {
                &self.all_lines[i]
            };
            buf.set_line(line_area.x, line_area.y, line, line_area.width);
        }
    }
}
