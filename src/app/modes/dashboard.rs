use crate::app::command::Command;
use crate::app::context::Context;
use crate::app::events::{ActionSections, GLOBAL, UserAction};
use crate::app::modes::reader::Reader;
use crate::app::modes::{Mode, ModeTrait};
use crate::components::Component;
use crate::components::splash_screen::SplashScreen;
use crate::prelude::*;
use ratatui::Frame;

pub struct Dashboard {
    pub splash: SplashScreen,
}

impl Dashboard {
    pub fn new() -> Self {
        Dashboard {
            splash: SplashScreen,
        }
    }
}

impl ModeTrait for Dashboard {
    fn update(mut self, ctx: &mut Context, event: &AppEvent) -> Result<Mode> {
        self.splash.update(event)?;

        match event {
            AppEvent::UserAction(UserAction::Quit) => return Ok(Mode::Exit),
            AppEvent::UserAction(UserAction::OpenReader) => {
                return Ok(Mode::Reader(Reader::new(ctx)?));
            }
            AppEvent::Command(Command::Goto { book, position }) => {
                ctx.session.book = book.clone();
                ctx.session.position = *position;
                return Ok(Mode::Reader(Reader::new(ctx)?));
            }
            _ => {}
        }
        Ok(Mode::Dashboard(self))
    }

    fn render(&mut self, _ctx: &Context, f: &mut Frame) -> Result<()> {
        let area = f.area();
        let buf = f.buffer_mut();

        self.splash.render(area, buf)?;
        Ok(())
    }

    fn actions(&self) -> ActionSections {
        &[GLOBAL, &[UserAction::OpenReader]]
    }
}
