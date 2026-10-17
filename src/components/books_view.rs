use crate::app::events::{AppEvent, UserAction};
use crate::chars::{Icon, Symbol};
use crate::components::Component;
use crate::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders};

/// Scrollbar thumb height in rows.
const THUMB_HEIGHT: usize = 5;
/// Rows scrolled by `HalfPageDown` and `HalfPageUp`.
const SCROLL_STEP: usize = 5;

pub struct BooksView {
    books: Vec<String>,
    selected_book_index: usize,
    scrolled_offset: usize,
    focused: bool,
    /// List height from the last render.
    visible: usize,
    /// Scroll the view to the selection on the next render.  Off while the list is scrolled
    /// without moving the selection.
    follow_selection: bool,

    // Pre-built cache lines to reduce cost of rendering.
    // TODO: Revisit this.
    unselected_lines: Vec<Line<'static>>,
    selected_lines: Vec<Line<'static>>,
}

impl BooksView {
    pub fn new(books: Vec<String>) -> Self {
        let all_lines = BooksView::build_line_cache(&books, "   ");
        let selected_prefix = format!(" {} ", Icon::Selected.as_str());
        let selected_lines = BooksView::build_line_cache(&books, &selected_prefix)
            .into_iter()
            .map(|l| l.style(theme::SELECTED))
            .collect();
        BooksView {
            books,
            selected_book_index: 0,
            scrolled_offset: 0,
            focused: false,
            visible: 0,
            follow_selection: true,
            unselected_lines: all_lines,
            selected_lines,
        }
    }

    pub fn selected_book(&self) -> &str {
        &self.books[self.selected_book_index]
    }

    /// Does nothing if the book does not exist.
    pub fn select(&mut self, book: &str) {
        if let Some(i) = self.books.iter().position(|b| b == book) {
            self.selected_book_index = i;
            self.follow_selection = true;
        }
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    /// Scrolls the view by `SCROLL_STEP` rows.  The selection, and so the open book, is kept.
    fn half_page(&mut self, down: bool) {
        let max_offset = self.books.len().saturating_sub(self.visible);
        self.scrolled_offset = match down {
            true => (self.scrolled_offset + SCROLL_STEP).min(max_offset),
            false => self.scrolled_offset.saturating_sub(SCROLL_STEP),
        };
        self.follow_selection = false;
    }

    /// Drawn over the right border, only when the list does not fit.  The thumb has a fixed
    /// length and moves from the top of the track to the bottom as the list scrolls.
    fn render_scrollbar(&self, area: Rect, style: Style, buf: &mut Buffer) {
        if self.books.len() <= self.visible || area.height < 3 {
            return;
        }
        let x = area.right() - 1;
        let track = (area.height - 2) as usize;
        let thumb = THUMB_HEIGHT.min(track);
        let max_offset = self.books.len() - self.visible;
        let start = self.scrolled_offset.min(max_offset) * (track - thumb) / max_offset;
        for row in start..start + thumb {
            buf[(x, area.y + 1 + row as u16)]
                .set_symbol(Symbol::ScrollbarThumb.as_str())
                .set_style(style);
        }
    }

    fn build_line_cache(books: &Vec<String>, prefix: &str) -> Vec<Line<'static>> {
        books
            .iter()
            .map(|b| Line::from(format!("{}{}", prefix, b)))
            .collect()
    }
}

impl Component for BooksView {
    fn update(&mut self, event: &AppEvent) -> Result<()> {
        match event {
            AppEvent::Focus => self.focused = true,
            AppEvent::Defocus => self.focused = false,
            AppEvent::UserAction(action) if self.focused => match action {
                UserAction::MoveDown => {
                    if self.selected_book_index < self.books.len() - 1 {
                        self.selected_book_index += 1;
                    }
                    self.follow_selection = true;
                }
                UserAction::MoveUp => {
                    if self.selected_book_index > 0 {
                        self.selected_book_index -= 1;
                    }
                    self.follow_selection = true;
                }
                UserAction::HalfPageDown => self.half_page(true),
                UserAction::HalfPageUp => self.half_page(false),
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" [1] Books ", theme::TITLE))
            .border_style(theme::border(self.focused));

        let inner = block.inner(area);
        let border_style = theme::border(self.focused);
        block.render(area, buf);

        // Scrolling logic.
        let visible = inner.height as usize;
        self.visible = visible;
        if self.follow_selection {
            let min_index = self.scrolled_offset;
            let max_index = self.scrolled_offset + visible - 1;
            if self.selected_book_index < min_index {
                self.scrolled_offset -= min_index - self.selected_book_index;
            }
            if self.selected_book_index > max_index {
                self.scrolled_offset += self.selected_book_index - max_index;
            }
        }

        self.render_scrollbar(area, border_style, buf);

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
                &self.unselected_lines[i]
            };
            buf.set_line(line_area.x, line_area.y, line, line_area.width);
        }
        Ok(())
    }
}
