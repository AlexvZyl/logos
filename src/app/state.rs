use crate::app::data::PersistentAppData;
use crate::app::state_dashboard::Dashboard;
use crate::app::state_default_reader::DefaultReader;
use crate::prelude::*;
use ratatui::Frame;

pub trait AppStateTrait {
    fn get_app_data(self) -> PersistentAppData;
    fn from_state(state: AppStateEnum) -> Result<AppStateEnum>;
    fn update(self, event: AppEvent) -> Result<AppStateEnum>;
    fn render(&mut self, f: &mut Frame) -> Result<()>;
}

pub enum AppStateEnum {
    Dashboard(Dashboard),
    DefaultReader(DefaultReader),
    Exit,
}

impl AppStateEnum {
    pub fn get_app_data(self) -> PersistentAppData {
        match self {
            AppStateEnum::Dashboard(s) => s.get_app_data(),
            AppStateEnum::DefaultReader(s) => s.get_app_data(),
            AppStateEnum::Exit => {
                panic!("Exit should not request data")
            }
        }
    }

    /// Whether keys should be sent raw, as `AppEvent::Key`, rather than mapped to actions.
    pub fn captures_input(&self) -> bool {
        match self {
            AppStateEnum::DefaultReader(s) => s.captures_input(),
            AppStateEnum::Dashboard(_) | AppStateEnum::Exit => false,
        }
    }

    pub fn update(self, event: AppEvent) -> Result<AppStateEnum> {
        match self {
            AppStateEnum::Dashboard(s) => s.update(event),
            AppStateEnum::DefaultReader(s) => s.update(event),
            AppStateEnum::Exit => {
                panic!("Exit should not update")
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame) -> Result<()> {
        match self {
            AppStateEnum::Dashboard(s) => s.render(f),
            AppStateEnum::DefaultReader(s) => s.render(f),
            AppStateEnum::Exit => {
                panic!("Exit should not render")
            }
        }
    }
}
