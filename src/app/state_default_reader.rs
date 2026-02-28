use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

use crate::app::events::UserAction;
use crate::app::data::AppData;
use crate::app::state::{AppStateEnum, AppStateTrait};
use crate::components::book_reader::BookReader;
use crate::components::books_view::BooksView;
use crate::components::footer::LogosFooter;
use crate::prelude::*;

pub enum SelectedWindow {
    BooksView,
    BookReader,
}

pub struct DefaultReader {
    app_data: AppData,
    selected_book_index: usize,
    scrolled_offset: usize,
    reader_scroll: u16,
    selected_window: SelectedWindow,
}

impl AppStateTrait for DefaultReader {
    fn from_state(state: AppStateEnum) -> Result<AppStateEnum> {
        let app_data = state.get_app_data();
        Ok(AppStateEnum::DefaultReader(DefaultReader {
            app_data,
            selected_book_index: 0,
            scrolled_offset: 0,
            reader_scroll: 0,
            selected_window: SelectedWindow::BooksView,
        }))
    }

    fn update(mut self, event: AppEvent) -> Result<AppStateEnum> {
        let book_count = self.app_data.bible.get_books().len();

        match event {
            AppEvent::AppStart => {}
            AppEvent::Tick(_) => {}
            AppEvent::UserAction(action) => match action {
                UserAction::Quit => return Ok(AppStateEnum::Exit),
                UserAction::MoveDown => {
                    if self.selected_book_index < book_count - 1 {
                        self.selected_book_index += 1;
                    }
                }
                UserAction::MoveUp => {
                    if self.selected_book_index > 0 {
                        self.selected_book_index -= 1;
                    }
                }
                UserAction::IncrementWindow => {
                    self.selected_window = match self.selected_window {
                        SelectedWindow::BooksView => SelectedWindow::BookReader,
                        SelectedWindow::BookReader => SelectedWindow::BooksView,
                    };
                }
            },
        }

        Ok(AppStateEnum::DefaultReader(self))
    }

    fn render(&mut self, f: &mut Frame) -> Result<()> {
        let [main, footer] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(f.area());

        let [books, content] =
            Layout::horizontal([Constraint::Percentage(15), Constraint::Fill(1)]).areas(main);

        let visible = (books.height - 3) as usize;

        let min_index = self.scrolled_offset;
        let max_index = self.scrolled_offset + visible;
        if self.selected_book_index < min_index {
            self.scrolled_offset -= min_index - self.selected_book_index;
        }
        if self.selected_book_index > max_index {
            self.scrolled_offset += self.selected_book_index - max_index;
        }

        f.render_widget(
            BooksView {
                books: self.app_data.bible.get_books(),
                selected_book_index: self.selected_book_index,
                scrolled_offset: self.scrolled_offset,
                focused: matches!(self.selected_window, SelectedWindow::BooksView),
            },
            books,
        );
        let book_name = &self.app_data.bible.get_books()[self.selected_book_index];
        f.render_widget(
            BookReader {
                bible: &self.app_data.bible,
                book: book_name,
                scroll_offset: self.reader_scroll,
                focused: matches!(self.selected_window, SelectedWindow::BookReader),
            },
            content,
        );
        f.render_widget(LogosFooter, footer);
        Ok(())
    }

    fn get_app_data(self) -> AppData {
        self.app_data
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
