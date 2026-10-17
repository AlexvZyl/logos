use crate::app::data::PersistentAppData;
use crate::app::events::{AppEvent, UserAction};
use crate::app::state::{AppStateEnum, AppStateTrait};
use crate::cache::{AppCache, ReaderCache};
use crate::components::Component;
use crate::components::book_column::Position;
use crate::components::book_reader::BookReader;
use crate::components::books_view::BooksView;
use crate::components::footer::LogosFooter;
use crate::components::references::References;
use crate::components::strongs::Strongs;
use crate::prelude::*;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq)]
pub enum FocusedWindow {
    Books,
    Reader,
    References,
    Strongs,
}

impl FocusedWindow {
    fn next(self) -> Self {
        match self {
            Self::Books => Self::Reader,
            Self::Reader => Self::References,
            Self::References => Self::Strongs,
            Self::Strongs => Self::Books,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Books => Self::Strongs,
            Self::Reader => Self::Books,
            Self::References => Self::Reader,
            Self::Strongs => Self::References,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DefaultReader {
    pub app_data: PersistentAppData,
    pub books_view: BooksView,
    pub book_reader: BookReader,
    pub footer: LogosFooter,
    pub references: References,
    pub strongs: Strongs,
    pub focused: FocusedWindow,
}

impl DefaultReader {
    fn defocus_all(&mut self) -> Result<()> {
        self.books_view.update(&AppEvent::Defocus)?;
        self.book_reader.update(&AppEvent::Defocus)?;
        self.references.update(&AppEvent::Defocus)?;
        self.strongs.update(&AppEvent::Defocus)?;
        Ok(())
    }

    fn focus(&mut self, window: FocusedWindow) -> Result<()> {
        self.defocus_all()?;
        self.focused = window;
        match window {
            FocusedWindow::Books => self.books_view.update(&AppEvent::Focus),
            FocusedWindow::Reader => self.book_reader.update(&AppEvent::Focus),
            FocusedWindow::References => self.references.update(&AppEvent::Focus),
            FocusedWindow::Strongs => self.strongs.update(&AppEvent::Focus),
        }
    }

    fn save_cache(&self) {
        let position = self.book_reader.position();
        let cache = AppCache {
            reader: ReaderCache {
                book: Some(self.book_reader.book_name().to_string()),
                chapter: position.chapter,
                verse: position.verse,
                num_columns: self.book_reader.num_columns(),
            },
        };
        if let Err(e) = cache.save() {
            warn!("Failed to save cache: {e}");
        }
    }
}

impl AppStateTrait for DefaultReader {
    fn from_state(state: AppStateEnum) -> Result<AppStateEnum> {
        let app_data = state.get_app_data();
        let books = app_data.bible.get_books().clone();
        let cache = AppCache::load().reader;

        let mut books_view = BooksView::new(books);
        if let Some(book) = &cache.book {
            books_view.select(book);
        }
        books_view.update(&AppEvent::Focus)?;

        let mut book_reader =
            BookReader::new(app_data.bible.clone(), books_view.selected_book().to_string());
        book_reader.set_num_columns(cache.num_columns);
        if cache.book.as_deref() == Some(books_view.selected_book()) {
            book_reader.set_position(Position {
                chapter: cache.chapter,
                verse: cache.verse,
            });
        }

        Ok(AppStateEnum::DefaultReader(DefaultReader {
            app_data,
            books_view,
            book_reader,
            footer: LogosFooter::new(),
            references: References::new(),
            strongs: Strongs::new(),
            focused: FocusedWindow::Books,
        }))
    }

    fn update(mut self, event: AppEvent) -> Result<AppStateEnum> {
        match &event {
            AppEvent::UserAction(UserAction::Quit) => {
                self.save_cache();
                return Ok(AppStateEnum::Exit);
            }
            AppEvent::UserAction(UserAction::IncrementWindow) => {
                self.focus(self.focused.next())?;
            }
            AppEvent::UserAction(UserAction::DecrementWindow) => {
                self.focus(self.focused.prev())?;
            }
            AppEvent::UserAction(UserAction::JumpToWindow(i)) => {
                self.focus(match i {
                    0 => FocusedWindow::Books,
                    1 => FocusedWindow::Reader,
                    2 => FocusedWindow::References,
                    3 => FocusedWindow::Strongs,
                    _ => self.focused,
                })?;
            }
            _ => {
                self.books_view.update(&event)?;
                self.book_reader.update(&event)?;
                self.references.update(&event)?;
                self.strongs.update(&event)?;
                self.footer.update(&event)?;
            }
        }

        self.book_reader.set_book(self.books_view.selected_book());
        Ok(AppStateEnum::DefaultReader(self))
    }

    fn render(&mut self, f: &mut Frame) -> Result<()> {
        let [main, footer] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(f.area());

        let [books, content, sidebar] = Layout::horizontal([
            Constraint::Percentage(15),
            Constraint::Fill(1),
            Constraint::Percentage(20),
        ])
        .areas(main);

        let [references, strongs] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(sidebar);

        let buf = f.buffer_mut();
        self.books_view.render(books, buf)?;
        self.book_reader.render(content, buf)?;
        self.references.render(references, buf)?;
        self.strongs.render(strongs, buf)?;
        self.footer.render(footer, buf)?;
        Ok(())
    }

    fn get_app_data(self) -> PersistentAppData {
        self.app_data
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
