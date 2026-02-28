use crate::app::data::AppData;
use crate::app::events::UserAction;
use crate::app::state::{AppStateEnum, AppStateTrait};
use crate::app::state_default_reader::DefaultReader;
use crate::components::splash_screen::SplashScreen;
use crate::prelude::*;
use ratatui::Frame;

pub struct StartupScreen {
    /// Optional so that we can have lazy loading.
    app_data: Option<AppData>,
    start: Instant,
}

impl StartupScreen {
    pub fn new() -> Self {
        StartupScreen {
            app_data: None,
            start: Instant::now(),
        }
    }
}

impl AppStateTrait for StartupScreen {
    fn from_state(_: AppStateEnum) -> Result<AppStateEnum> {
        panic!("Should never go from a state to OpeningState");
    }

    fn update(mut self, event: AppEvent) -> Result<AppStateEnum> {
        match event {
            // TODO: Async would be cool here.
            AppEvent::AppStart => {
                self.app_data = Some(AppData::from_translation("KVJ")?);
            }
            // Keep splash screen up for a short while.
            AppEvent::Tick(_) => {
                if self.start.elapsed() > MIN_SPLASH_SCREEN_TIME {
                    return DefaultReader::from_state(AppStateEnum::Opening(self));
                }
            }
            AppEvent::UserAction(action) => match action {
                UserAction::Quit => return Ok(AppStateEnum::Exit),
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
