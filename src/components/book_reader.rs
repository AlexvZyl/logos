use ratatui::layout::Direction;
use ratatui::widgets::{Block, BorderType, Borders};

use crate::app::events::{AppEvent, UserAction};
use crate::bible::Bible;
use crate::components::book_column::Column;
use crate::components::Component;
use crate::prelude::*;

pub struct BookReader {
    bible: Arc<Bible>,
    current_book_name: String,
    scrolled_offset: usize,
    focused: bool,
    book_changed: bool,
    num_columns: usize,
    // For lazy loading.
    column: Option<Column>,
}

impl BookReader {
    pub fn new(bible: Arc<Bible>, current_book_name: String) -> Self {
        BookReader {
            bible,
            current_book_name,
            scrolled_offset: 0,
            focused: false,
            book_changed: true,
            column: None,
            // TODO: Change based on terminal size.
            num_columns: 2,
        }
    }

    pub fn set_book(&mut self, book: &str) {
        if self.current_book_name != book {
            self.current_book_name = book.to_string();
            self.scrolled_offset = 0;
            self.book_changed = true;
        }
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

    /// TODO: Revisit.
    fn build_column(&self, width: usize, height: usize) -> Result<Column> {
        let (first, _) = Column::from(
            width,
            height,
            self.bible.as_ref(),
            &self.current_book_name,
            1,
            None,
        )?;
        Ok(first)
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
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let title = format!(" [2] {} ", self.current_book_name);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title.yellow().bold())
            .border_style(if self.focused {
                Style::default().blue()
            } else {
                Style::default()
            });

        let padded = Self::layout(block.inner(area), self.scrolled_offset, self.num_columns);

        block.render(area, buf);

        if self.column.is_none() || self.book_changed {
            self.column = Some(self.build_column(padded.width as usize, padded.height as usize)?);
            self.book_changed = false;
        }
        self.column.as_mut().unwrap().render(padded, buf)?;
        Ok(())
    }
}
