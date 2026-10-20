use crate::app::command::Command;
use crate::app::context::Context;
use crate::app::events::{
    ActionSections, AppEvent, COLUMNS, GLOBAL, NAVIGATION, UserAction, WINDOWS,
};
use crate::app::modes::{Mode, ModeTrait};
use crate::components::Component;
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
enum Panel {
    Books,
    Reader,
    References,
    Strongs,
}

impl Panel {
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

pub struct Reader {
    pub books_view: BooksView,
    pub book_reader: BookReader,
    pub footer: LogosFooter,
    pub references: References,
    pub strongs: Strongs,
    focused: Panel,
}

impl Reader {
    pub fn new(ctx: &Context) -> Result<Reader> {
        let session = &ctx.session;

        let mut books_view = BooksView::new(ctx.bible.book_groups());
        books_view.select(&session.book);
        assert!(*books_view.selected_book() == session.book);
        books_view.update(&AppEvent::Focus)?;

        let mut book_reader = BookReader::new(ctx.bible.clone(), session.book.clone());
        book_reader.set_num_columns(session.num_columns);
        book_reader.set_position(session.position);

        Ok(Reader {
            books_view,
            book_reader,
            footer: LogosFooter::new(),
            references: References::new(),
            strongs: Strongs::new(),
            focused: Panel::Books,
        })
    }

    fn defocus_all(&mut self) -> Result<()> {
        self.books_view.update(&AppEvent::Defocus)?;
        self.book_reader.update(&AppEvent::Defocus)?;
        self.references.update(&AppEvent::Defocus)?;
        self.strongs.update(&AppEvent::Defocus)?;
        Ok(())
    }

    fn focus(&mut self, panel: Panel) -> Result<()> {
        self.defocus_all()?;
        self.focused = panel;
        match panel {
            Panel::Books => self.books_view.update(&AppEvent::Focus),
            Panel::Reader => self.book_reader.update(&AppEvent::Focus),
            Panel::References => self.references.update(&AppEvent::Focus),
            Panel::Strongs => self.strongs.update(&AppEvent::Focus),
        }
    }

    /// Writes where the user is back to the session, so other modes and the cache see it.
    fn sync_session(&self, ctx: &mut Context) {
        let session = &mut ctx.session;
        if session.book != *self.book_reader.book() {
            session.book = self.book_reader.book().clone();
        }
        session.position = self.book_reader.position();
        session.num_columns = self.book_reader.num_columns();
    }
}

impl ModeTrait for Reader {
    fn update(mut self, ctx: &mut Context, event: &AppEvent) -> Result<Mode> {
        match event {
            AppEvent::UserAction(UserAction::Quit) => {
                self.sync_session(ctx);
                return Ok(Mode::Exit);
            }
            AppEvent::UserAction(UserAction::IncrementWindow) => {
                self.focus(self.focused.next())?;
            }
            AppEvent::UserAction(UserAction::DecrementWindow) => {
                self.focus(self.focused.prev())?;
            }
            AppEvent::UserAction(UserAction::JumpToWindow(i)) => {
                self.focus(match i {
                    0 => Panel::Books,
                    1 => Panel::Reader,
                    2 => Panel::References,
                    3 => Panel::Strongs,
                    _ => self.focused,
                })?;
            }
            AppEvent::Command(Command::Goto { book, position }) => {
                self.books_view.select(book);
                self.book_reader.set_book(book);
                // After `set_book`, which resets the position when the book changes.
                self.book_reader.set_position(*position);
            }
            // Turn the reader's pages while browsing books.
            AppEvent::UserAction(action @ (UserAction::NextPage | UserAction::PrevPage))
                if self.focused == Panel::Books =>
            {
                self.book_reader.page(*action);
            }
            _ => {
                self.books_view.update(event)?;
                self.book_reader.update(event)?;
                self.references.update(event)?;
                self.strongs.update(event)?;
                self.footer.update(event)?;
            }
        }

        self.book_reader.set_book(self.books_view.selected_book());
        self.sync_session(ctx);
        Ok(Mode::Reader(self))
    }

    fn render(&mut self, _ctx: &Context, f: &mut Frame) -> Result<()> {
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

    fn actions(&self) -> ActionSections {
        match self.focused {
            Panel::Books => &[GLOBAL, WINDOWS, NAVIGATION],
            Panel::Reader => &[GLOBAL, WINDOWS, NAVIGATION, COLUMNS],
            Panel::References | Panel::Strongs => &[GLOBAL, WINDOWS],
        }
    }

    fn confirm_quit(&self) -> bool {
        true
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
