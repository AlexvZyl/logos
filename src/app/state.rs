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
    DefaultReader(DefaultReaderViewState),
}

impl AppStateEnum {
    pub fn update(self, event: Event) -> Result<AppStateEnum> {
        match self {
            AppStateEnum::Opening(s) => s.update(event),
            AppStateEnum::DefaultReader(s) => s.update(event),
        }
    }
    pub fn render(&self, f: &mut Frame) -> Result<()> {
        match self {
            AppStateEnum::Opening(s) => s.render(f),
            AppStateEnum::DefaultReader(s) => s.render(f),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum Event {
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
        self.app_data = Some(AppData::from_translation("KVJ")?);
        match event {
            Event::Tick(_) => {
                if self.start.elapsed() > Duration::from_millis(1000) {
                    return DefaultReaderViewState::from_state(AppStateEnum::Opening(self));
                }
            }
            Event::KeyPress(_) => {}
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

pub struct DefaultReaderViewState {
    app_data: AppData,
}

impl AppState for DefaultReaderViewState {
    fn from_state(state: AppStateEnum) -> Result<AppStateEnum> {
        let app_data = match state {
            AppStateEnum::Opening(s) => s.get_app_data(),
            AppStateEnum::DefaultReader(s) => s.get_app_data(),
        };
        Ok(AppStateEnum::DefaultReader(DefaultReaderViewState {
            app_data,
        }))
    }

    fn update(self, event: Event) -> Result<AppStateEnum> {
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
