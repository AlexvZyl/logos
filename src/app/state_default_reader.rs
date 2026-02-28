use crate::app::data::AppData;
use crate::app::events::{AppEvent, UserAction};
use crate::app::state::{AppStateEnum, AppStateTrait};
use crate::components::Component;
use crate::components::book_reader::BookReader;
use crate::components::books_view::BooksView;
use crate::components::footer::LogosFooter;
use crate::prelude::*;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

pub struct DefaultReader {
    app_data: AppData,
    books_view: BooksView,
    book_reader: BookReader,
    footer: LogosFooter,
}

impl AppStateTrait for DefaultReader {
    fn from_state(state: AppStateEnum) -> Result<AppStateEnum> {
        let app_data = state.get_app_data();
        let books = app_data.bible.get_books().clone();
        // TODO: Read from cache.
        let initial_book = &books[0];

        let book_reader = BookReader::new(&app_data.bible, initial_book);
        let mut books_view = BooksView::new(books);
        books_view.update(&AppEvent::Focus);

        Ok(AppStateEnum::DefaultReader(DefaultReader {
            app_data,
            books_view,
            book_reader,
            footer: LogosFooter,
        }))
    }

    fn update(mut self, event: AppEvent) -> Result<AppStateEnum> {
        match &event {
            AppEvent::UserAction(UserAction::Quit) => return Ok(AppStateEnum::Exit),
            AppEvent::UserAction(UserAction::IncrementWindow) => {
                if self.books_view.focused() {
                    self.books_view.update(&AppEvent::Defocus);
                    self.book_reader.update(&AppEvent::Focus);
                } else {
                    self.book_reader.update(&AppEvent::Defocus);
                    self.books_view.update(&AppEvent::Focus);
                }
            }
            AppEvent::UserAction(UserAction::MoveDown | UserAction::MoveUp) => {
                self.books_view.update(&event);
                self.book_reader
                    .set_book(&self.app_data.bible, self.books_view.selected_book());
                self.book_reader.update(&event);
            }
            _ => {
                self.books_view.update(&event);
                self.book_reader.update(&event);
                self.footer.update(&event);
            }
        }

        Ok(AppStateEnum::DefaultReader(self))
    }

    fn render(&mut self, f: &mut Frame) -> Result<()> {
        let [main, footer] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(f.area());

        let [books, content] =
            Layout::horizontal([Constraint::Percentage(15), Constraint::Fill(1)]).areas(main);

        let buf = f.buffer_mut();
        self.books_view.render(books, buf);
        self.book_reader.render(content, buf);
        self.footer.render(footer, buf);
        Ok(())
    }

    fn get_app_data(self) -> AppData {
        self.app_data
    }
}
