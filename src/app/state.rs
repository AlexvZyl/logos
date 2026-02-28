use std::time::{Duration, Instant};

use crate::app::data::AppData;
use crate::components::books_view::BooksView;
use crate::components::footer::LogosFooter;
use crate::components::splash_screen::SplashScreen;
use crate::prelude::*;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait AppState {
    fn get_app_data(self) -> AppData;
    fn from_state(state: AppStateEnum) -> Result<AppStateEnum>;
    fn update(self, event: Event) -> Result<AppStateEnum>;
    fn render(&mut self, f: &mut Frame) -> Result<()>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum AppStateEnum {
    Opening(OpeningState),
    DefaultReader(DefaultReaderState),
    Exit,
}

impl AppStateEnum {
    pub fn get_app_data(self) -> AppData {
        match self {
            AppStateEnum::Opening(s) => s.get_app_data(),
            AppStateEnum::DefaultReader(s) => s.get_app_data(),
            AppStateEnum::Exit => {
                panic!("Should not reach here")
            }
        }
    }

    pub fn update(self, event: Event) -> Result<AppStateEnum> {
        match self {
            AppStateEnum::Opening(s) => s.update(event),
            AppStateEnum::DefaultReader(s) => s.update(event),
            AppStateEnum::Exit => {
                panic!("Should not reach here")
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame) -> Result<()> {
        match self {
            AppStateEnum::Opening(s) => s.render(f),
            AppStateEnum::DefaultReader(s) => s.render(f),
            AppStateEnum::Exit => {
                panic!("Should not reach here")
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum Event {
    AppStart,
    // Time in ms.
    Tick(usize),
    // The pressed key.
    KeyPress(char),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct OpeningState {
    /// Optional so that we can have lazy loading.
    app_data: Option<AppData>,
    start: Instant,
}

impl OpeningState {
    pub fn new() -> Self {
        OpeningState {
            app_data: None,
            start: Instant::now(),
        }
    }
}

impl AppState for OpeningState {
    fn from_state(_: AppStateEnum) -> Result<AppStateEnum> {
        panic!("Should never go from a state to OpeningState");
    }

    fn update(mut self, event: Event) -> Result<AppStateEnum> {
        match event {
            // Lazy load data.
            // TODO: Async would be cool here.
            Event::AppStart => {
                self.app_data = Some(AppData::from_translation("KVJ")?);
            }
            // Keep splash screen up for a short while.
            Event::Tick(_) => {
                if self.start.elapsed() > Duration::from_millis(1000) {
                    return DefaultReaderState::from_state(AppStateEnum::Opening(self));
                }
            }
            Event::KeyPress(c) => match c {
                'q' => return Ok(AppStateEnum::Exit),
                _ => {}
            },
        }
        return Ok(AppStateEnum::Opening(self));
    }

    fn render(&mut self, f: &mut Frame) -> Result<()> {
        f.render_widget(SplashScreen, f.area());
        Ok(())
    }

    fn get_app_data(self) -> AppData {
        self.app_data
            .expect("Should not change state if app data not loaded")
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DefaultReaderState {
    app_data: AppData,
    selected_book_index: usize,
    scrolled_offset: usize,
}

impl AppState for DefaultReaderState {
    fn from_state(state: AppStateEnum) -> Result<AppStateEnum> {
        let app_data = state.get_app_data();
        Ok(AppStateEnum::DefaultReader(DefaultReaderState {
            app_data,
            selected_book_index: 0,
            scrolled_offset: 0,
        }))
    }

    fn update(mut self, event: Event) -> Result<AppStateEnum> {
        let book_count = self.app_data.bible.get_books().len();

        match event {
            Event::AppStart => {}
            Event::Tick(_) => {}
            Event::KeyPress(c) => match c {
                'q' => return Ok(AppStateEnum::Exit),
                'j' => {
                    if self.selected_book_index < book_count - 1 {
                        self.selected_book_index += 1;
                    }
                }
                'k' => {
                    if self.selected_book_index > 0 {
                        self.selected_book_index -= 1;
                    }
                }
                _ => {}
            },
        }

        Ok(AppStateEnum::DefaultReader(self))
    }

    fn render(&mut self, f: &mut Frame) -> Result<()> {
        let [main, footer] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(f.area());

        let [books, _content] =
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
            },
            books,
        );
        f.render_widget(LogosFooter, footer);
        Ok(())
    }

    fn get_app_data(self) -> AppData {
        self.app_data
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
