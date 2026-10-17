use crate::app::events::{AppEvent, UserAction};
use crate::chars::{Icon, Symbol};
use crate::components::Component;
use crate::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders};
use unicode_width::UnicodeWidthStr;

/// Scrollbar thumb height in rows.
const THUMB_HEIGHT: usize = 5;
/// Rows scrolled by `HalfPageDown` and `HalfPageUp`.
const SCROLL_STEP: usize = 5;

/// A row of the list.
enum Row {
    /// Separator with the group title, e.g. `Old Testament`.
    Group(String),
    /// Index into `books`.
    Book(usize),
}

pub struct BooksView {
    books: Vec<String>,
    rows: Vec<Row>,
    /// Row of each book.
    book_rows: Vec<usize>,
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
    /// `groups` is `(title, books)`.  Groups without a title get no separator.
    pub fn new(groups: Vec<(String, Vec<String>)>) -> Self {
        let mut books = Vec::new();
        let mut rows = Vec::new();
        let mut book_rows = Vec::new();
        for (title, names) in groups {
            if !title.is_empty() {
                rows.push(Row::Group(title));
            }
            for name in names {
                book_rows.push(rows.len());
                rows.push(Row::Book(books.len()));
                books.push(name);
            }
        }
        assert!(!books.is_empty());
        assert_eq!(book_rows.len(), books.len());
        let all_lines = BooksView::build_line_cache(&books, "   ");
        let selected_prefix = format!(" {} ", Icon::Selected.as_str());
        let selected_lines = BooksView::build_line_cache(&books, &selected_prefix)
            .into_iter()
            .map(|l| l.style(theme::SELECTED))
            .collect();
        BooksView {
            books,
            rows,
            book_rows,
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
        let max_offset = self.rows.len().saturating_sub(self.visible);
        self.scrolled_offset = match down {
            true => (self.scrolled_offset + SCROLL_STEP).min(max_offset),
            false => self.scrolled_offset.saturating_sub(SCROLL_STEP),
        };
        self.follow_selection = false;
    }

    /// Drawn over the right border, only when the list does not fit.  The thumb has a fixed
    /// length and moves from the top of the track to the bottom as the list scrolls.
    fn render_scrollbar(&self, area: Rect, style: Style, buf: &mut Buffer) {
        if self.rows.len() <= self.visible || area.height < 3 {
            return;
        }
        let x = area.right() - 1;
        let track = (area.height - 2) as usize;
        let thumb = THUMB_HEIGHT.min(track);
        let max_offset = self.rows.len() - self.visible;
        let start = self.scrolled_offset.min(max_offset) * (track - thumb) / max_offset;
        assert!(start + thumb <= track);
        for row in start..start + thumb {
            buf[(x, area.y + 1 + row as u16)]
                .set_symbol(Symbol::ScrollbarThumb.as_str())
                .set_style(style);
        }
    }

    /// `── Old Testament ─────`, filled to `width`.
    fn separator(title: &str, width: usize) -> Line<'static> {
        let line = Symbol::HorizontalLine.as_str();
        let label = format!("{line}{line} {title} ");
        let fill = width.saturating_sub(label.width());
        Line::styled(format!("{label}{}", line.repeat(fill)), theme::MUTED)
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
        // Nothing is visible to follow at zero height.
        if self.follow_selection && visible > 0 {
            let row = self.book_rows[self.selected_book_index];
            // Show the separator above the first book of a group.
            let top = match row.checked_sub(1).map(|r| &self.rows[r]) {
                Some(Row::Group(_)) => row - 1,
                _ => row,
            };
            self.scrolled_offset = self.scrolled_offset.min(top);
            self.scrolled_offset = self.scrolled_offset.max((row + 1).saturating_sub(visible));
            let view = self.scrolled_offset..self.scrolled_offset + visible;
            assert!(view.contains(&row));
        }
        // Growing the terminal would otherwise leave blank rows below the last book.
        self.scrolled_offset = self.scrolled_offset.min(self.rows.len().saturating_sub(visible));

        self.render_scrollbar(area, border_style, buf);

        let width = inner.width as usize;
        let rows = self.rows.iter().skip(self.scrolled_offset).take(visible);
        for (y, row) in (inner.y..).zip(rows) {
            match *row {
                Row::Group(ref title) => {
                    let line = Self::separator(title, width);
                    buf.set_line(inner.x, y, &line, inner.width);
                }
                Row::Book(i) => {
                    let line = match i == self.selected_book_index {
                        true => &self.selected_lines[i],
                        false => &self.unselected_lines[i],
                    };
                    buf.set_line(inner.x, y, line, inner.width);
                }
            }
        }
        Ok(())
    }
}
