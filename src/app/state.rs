use crate::app::actions::UserAction;
use crate::app::data::AppData;
use crate::app::state_default_reader::DefaultReader;
use crate::app::state_startup_screen::StartupScreen;
use crate::prelude::*;
use ratatui::Frame;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait AppStateTrait {
    fn get_app_data(self) -> AppData;
    fn from_state(state: AppStateEnum) -> Result<AppStateEnum>;
    fn update(self, event: AppEvent) -> Result<AppStateEnum>;
    fn render(&mut self, f: &mut Frame) -> Result<()>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum AppStateEnum {
    Opening(StartupScreen),
    DefaultReader(DefaultReader),
    Exit,
}

impl AppStateEnum {
    pub fn get_app_data(self) -> AppData {
        match self {
            AppStateEnum::Opening(s) => s.get_app_data(),
            AppStateEnum::DefaultReader(s) => s.get_app_data(),
            AppStateEnum::Exit => {
                panic!("Exit should not request data")
            }
        }
    }

    pub fn update(self, event: AppEvent) -> Result<AppStateEnum> {
        match self {
            AppStateEnum::Opening(s) => s.update(event),
            AppStateEnum::DefaultReader(s) => s.update(event),
            AppStateEnum::Exit => {
                panic!("Exit should not update")
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame) -> Result<()> {
        match self {
            AppStateEnum::Opening(s) => s.render(f),
            AppStateEnum::DefaultReader(s) => s.render(f),
            AppStateEnum::Exit => {
                panic!("Exit should not render")
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum AppEvent {
    /// Special event fired at start of app.
    AppStart,
    /// Time in ms since last tick.
    Tick(usize),
    /// Action performed/requested by the user.
    UserAction(UserAction),
}
