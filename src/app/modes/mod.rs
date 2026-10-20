pub mod dashboard;
pub mod reader;

use crate::app::context::Context;
use crate::app::events::ActionSections;
use crate::prelude::*;
use dashboard::Dashboard;
use ratatui::Frame;
use reader::Reader;

pub trait ModeTrait {
    /// Consumes the mode and returns the next one, which may be the same mode.
    fn update(self, ctx: &mut Context, event: &AppEvent) -> Result<Mode>;
    fn render(&mut self, ctx: &Context, f: &mut Frame) -> Result<()>;
    /// Available actions, grouped into help sections in display order.
    fn actions(&self) -> ActionSections;
}

/// What the whole app is showing.
pub enum Mode {
    Dashboard(Dashboard),
    Reader(Reader),
    /// The app loop stops as soon as it sees this, so it is never updated or rendered.
    Exit,
}

impl Mode {
    pub fn update(self, ctx: &mut Context, event: &AppEvent) -> Result<Mode> {
        match self {
            Mode::Dashboard(m) => m.update(ctx, event),
            Mode::Reader(m) => m.update(ctx, event),
            Mode::Exit => unreachable!(),
        }
    }

    pub fn render(&mut self, ctx: &Context, f: &mut Frame) -> Result<()> {
        match self {
            Mode::Dashboard(m) => m.render(ctx, f),
            Mode::Reader(m) => m.render(ctx, f),
            Mode::Exit => unreachable!(),
        }
    }

    pub fn actions(&self) -> ActionSections {
        match self {
            Mode::Dashboard(m) => m.actions(),
            Mode::Reader(m) => m.actions(),
            Mode::Exit => unreachable!(),
        }
    }
}
