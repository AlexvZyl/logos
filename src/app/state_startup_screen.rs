use crate::app::data::PersistentAppData;
use crate::app::events::UserAction;
use crate::app::state::{AppStateEnum, AppStateTrait};
use crate::app::state_default_reader::DefaultReader;
use crate::components::Component;
use crate::components::footer::LogosFooter;
use crate::components::splash_screen::SplashScreen;
use crate::prelude::*;
use ratatui::Frame;

pub struct StartupScreen {
    pub app_data: Option<PersistentAppData>,
    pub start: Instant,
    pub splash: SplashScreen,
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
                self.app_data = Some(PersistentAppData::from_translation("KVJ")?);
            }
            AppEvent::UserAction(action) => match action {
                UserAction::Quit => return Ok(AppStateEnum::Exit),
                _ => {}
            },
            _ => {}
        }
        return DefaultReader::from_state(AppStateEnum::Opening(self));
    }

    fn render(&mut self, f: &mut Frame) -> Result<()> {
        let area = f.area();
        let buf = f.buffer_mut();

        let [main, footer] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        self.splash.render(main, buf);
        LogosFooter::new().render(footer, buf);
        Ok(())
    }

    fn get_app_data(self) -> PersistentAppData {
        self.app_data
            .expect("Should not change state if app data not loaded")
    }
}
