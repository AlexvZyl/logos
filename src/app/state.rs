use std::time::{Duration, Instant};

use crate::app::data::AppData;
use crate::components::splash_screen::SplashScreen;
use crate::prelude::*;
use ratatui::Frame;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait AppState {
    fn get_app_data(self) -> AppData;
    fn from_state(state: AppStateEnum) -> Result<AppStateEnum>;
    fn update(self, event: Event) -> Result<AppStateEnum>;
    fn render(&self, f: &mut Frame) -> Result<()>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum AppStateEnum {
    Opening(OpeningState),
    DefaultReader(DefaultReaderState),
    Exit,
}

impl AppStateEnum {
    pub fn update(self, event: Event) -> Result<AppStateEnum> {
        match self {
            AppStateEnum::Opening(s) => s.update(event),
            AppStateEnum::DefaultReader(s) => s.update(event),
            AppStateEnum::Exit => {
                panic!("Should not reach here")
            }
        }
    }
    pub fn render(&self, f: &mut Frame) -> Result<()> {
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
                if self.start.elapsed() > Duration::from_millis(500) {
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

    fn render(&self, f: &mut Frame) -> Result<()> {
        SplashScreen.render(f.area(), f.buffer_mut());
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
}

impl AppState for DefaultReaderState {
    fn from_state(state: AppStateEnum) -> Result<AppStateEnum> {
        let app_data = match state {
            AppStateEnum::Opening(s) => s.get_app_data(),
            AppStateEnum::DefaultReader(s) => s.get_app_data(),
            AppStateEnum::Exit => panic!("Should not be here"),
        };
        Ok(AppStateEnum::DefaultReader(DefaultReaderState { app_data }))
    }

    fn update(self, event: Event) -> Result<AppStateEnum> {
        match event {
            Event::AppStart => {}
            Event::Tick(_) => {}
            Event::KeyPress(c) => match c {
                'q' => return Ok(AppStateEnum::Exit),
                _ => {}
            },
        }
        Ok(AppStateEnum::DefaultReader(self))
    }

    fn render(&self, f: &mut Frame) -> Result<()> {
        Ok(())
    }

    fn get_app_data(self) -> AppData {
        self.app_data
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
