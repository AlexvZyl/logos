use ratatui::layout::Direction;
use ratatui::widgets::{Block, BorderType, Borders};

use crate::app::events::{AppEvent, UserAction};
use crate::bible::Bible;
use crate::components::Component;
use crate::components::book_column::{BookLayout, PAGE_GAP, Position};
use crate::prelude::*;
use unicode_width::UnicodeWidthStr;

const MAX_COLUMNS: usize = 4;

pub struct BookReader {
    bible: Arc<Bible>,
    current_book_name: String,
    scrolled_offset: usize,
    focused: bool,
    book_changed: bool,
    num_columns: usize,
    /// Column height from the last render.
    page_height: usize,
    // For lazy loading.
    book_layout: Option<BookLayout>,
    /// Stable reading position, used to restore the offset when the layout changes.
    position: Position,
    /// `(width, height, num_columns)` from the last render.  `None` forces a restore.
    last_params: Option<(usize, usize, usize)>,
    last_offset: usize,
}

impl BookReader {
    pub fn new(bible: Arc<Bible>, current_book_name: String) -> Self {
        BookReader {
            bible,
            current_book_name,
            scrolled_offset: 0,
            focused: false,
            book_changed: true,
            book_layout: None,
            // TODO: Change based on terminal size.
            num_columns: 2,
            page_height: 0,
            position: Position::default(),
            last_params: None,
            last_offset: 0,
        }
    }

    pub fn book_name(&self) -> &str {
        &self.current_book_name
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn num_columns(&self) -> usize {
        self.num_columns
    }

    /// Takes effect on the next render, once the layout is known.
    pub fn set_position(&mut self, position: Position) {
        self.position = position;
        self.last_params = None;
    }

    /// Whether the current book has `position`.  Verse `0` is the chapter heading.
    pub fn contains(&self, position: Position) -> Result<bool> {
        let book = self.bible.get_book(&self.current_book_name)?;
        Ok(book.get_chapters().find(|c| c.number == position.chapter).is_some_and(|c| {
            position.verse == 0 || c.get_verses().any(|v| v.number == position.verse)
        }))
    }

    pub fn set_book(&mut self, book: &str) {
        if self.current_book_name != book {
            self.current_book_name = book.to_string();
            self.scrolled_offset = 0;
            self.book_changed = true;
            self.set_position(Position::default());
        }
    }

    /// Handles `NextPage`, `PrevPage`, `HalfPageDown` and `HalfPageUp`, ignores anything else.
    /// Public so that paging works while another window is focused.
    pub fn page(&mut self, action: UserAction) {
        let stride = self.page_height + PAGE_GAP;
        let half = self.page_height / 2;
        self.scrolled_offset = match action {
            UserAction::NextPage => (self.scrolled_offset / stride + 1) * stride,
            UserAction::PrevPage => self.scrolled_offset.saturating_sub(1) / stride * stride,
            UserAction::HalfPageDown => self.scrolled_offset.saturating_add(half),
            UserAction::HalfPageUp => self.scrolled_offset.saturating_sub(half),
            _ => self.scrolled_offset,
        };
    }

    pub fn set_num_columns(&mut self, num_columns: usize) {
        self.num_columns = num_columns.clamp(1, MAX_COLUMNS);
    }

    /// TODO: Simulate scrollling.
    fn layout(area: Rect, _scrolled_offset: usize, _num_columns: usize) -> Rect {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(2),
            ])
            .split(area)[1]
    }

    fn columns(area: Rect, num_columns: usize) -> Vec<Rect> {
        const GAP: u16 = 2;
        let n = num_columns.max(1) as u16;
        let width = area.width.saturating_sub(GAP * (n - 1)) / n;
        Layout::horizontal(vec![Constraint::Length(width); n as usize])
            .spacing(GAP)
            .split(area)
            .to_vec()
    }
}

impl Component for BookReader {
    fn update(&mut self, event: &AppEvent) -> Result<()> {
        match event {
            AppEvent::Focus => self.focused = true,
            AppEvent::Defocus => self.focused = false,
            AppEvent::UserAction(action) if self.focused => match action {
                UserAction::MoveDown => {
                    self.scrolled_offset = self.scrolled_offset.saturating_add(1);
                }
                UserAction::MoveUp => {
                    self.scrolled_offset = self.scrolled_offset.saturating_sub(1);
                }
                UserAction::NextPage
                | UserAction::PrevPage
                | UserAction::HalfPageDown
                | UserAction::HalfPageUp => self.page(*action),
                UserAction::IncrementColumns => self.set_num_columns(self.num_columns + 1),
                UserAction::DecrementColumns => self.set_num_columns(self.num_columns.saturating_sub(1)),
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let title = format!(" [2] {} ", self.current_book_name);
        let title_width = title.width();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(title, theme::TITLE))
            .border_style(theme::border(self.focused));

        let padded = Self::layout(block.inner(area), self.scrolled_offset, self.num_columns);

        let columns = Self::columns(padded, self.num_columns);
        assert_eq!(columns.len(), self.num_columns);
        let width = columns[0].width as usize;
        if self.book_changed || self.book_layout.as_ref().is_none_or(|l| l.width != width) {
            let book = self.bible.get_book(&self.current_book_name)?;
            self.book_layout = Some(BookLayout::new(&self.bible, book, width));
            self.book_changed = false;
        }
        let layout = self.book_layout.as_ref().expect("Just created the layout");

        let height = padded.height as usize;
        self.page_height = height;
        let n = columns.len();

        let params = Some((width, height, n));
        if self.last_params != params {
            self.scrolled_offset = layout.offset_of(self.position, n, height);
            self.last_offset = self.scrolled_offset;
            self.last_params = params;
        }
        self.scrolled_offset = self.scrolled_offset.min(layout.max_offset(n, height));
        // Only update the position on scroll, so that restoring does not drift it.
        if self.scrolled_offset != self.last_offset {
            self.position = layout.position_at(self.scrolled_offset, n, height);
            self.last_offset = self.scrolled_offset;
        }

        let screens = layout.screen_count(n, height).max(1);
        let screen = layout.screen_at(self.scrolled_offset, height) + 1;
        // Padded to the width of the total, so the title does not shift while paging.
        let digits = screens.to_string().len();
        let screen = screen.min(screens);
        let page = format!(" [{screen:0digits$}/{screens}] ");
        let translation = format!(" [{}] ", self.bible.name());
        // Centred on the whole border, hidden when it would overlap the other titles.
        let side = title_width.max(page.width()) + 1;
        let block = match 2 * side + translation.width() <= area.width as usize {
            true => block.title_top(Line::styled(translation, theme::MUTED).centered()),
            false => block,
        };
        block
            .title_top(Line::styled(page, theme::MUTED).right_aligned())
            .render(area, buf);

        for (i, column) in columns.iter().enumerate() {
            let rows = layout.column(i, n, self.scrolled_offset, height);
            Paragraph::new(rows).render(*column, buf);
        }

        Ok(())
    }
}
