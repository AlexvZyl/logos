use crate::app::data::AppData;
use crate::app::events::UserAction;
use crate::app::state::{AppStateEnum, AppStateTrait};
use crate::app::state_default_reader::DefaultReader;
use crate::components::Component;
use crate::components::splash_screen::SplashScreen;
use crate::prelude::*;
use ratatui::Frame;

pub struct StartupScreen {
    app_data: Option<AppData>,
    start: Instant,
    splash: SplashScreen,
}

impl StartupScreen {
    pub fn new() -> Self {
        StartupScreen {
            app_data: None,
            start: Instant::now(),
            splash: SplashScreen,
        }
    }
}

impl AppStateTrait for StartupScreen {
    fn from_state(_: AppStateEnum) -> Result<AppStateEnum> {
        panic!("Should never go from a state to OpeningState");
    }

    fn update(mut self, event: AppEvent) -> Result<AppStateEnum> {
        self.splash.update(&event);

        match event {
            AppEvent::AppStart => {
                self.app_data = Some(AppData::from_translation("KVJ")?);
            }
            AppEvent::Tick(_) => {
                if self.start.elapsed() > MIN_SPLASH_SCREEN_TIME {
                    return DefaultReader::from_state(AppStateEnum::Opening(self));
                }
            }
            AppEvent::UserAction(action) => match action {
                UserAction::Quit => return Ok(AppStateEnum::Exit),
                _ => {}
            },
            AppEvent::Focus | AppEvent::Defocus => {}
        }
        return Ok(AppStateEnum::Opening(self));
    }

    fn render(&mut self, f: &mut Frame) -> Result<()> {
        self.splash.render(f.area(), f.buffer_mut());
        Ok(())
    }

    fn get_app_data(self) -> AppData {
        self.app_data
            .expect("Should not change state if app data not loaded")
    }
}
