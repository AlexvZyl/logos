#![allow(dead_code)]

mod app;
mod bible;
mod components;
mod config;
mod error;
mod filesystem;
mod prelude;

use crate::app::events::KeyMap;
use crate::app::state::AppStateEnum;
use crate::app::state_startup_screen::StartupScreen;
use crate::prelude::*;
use crossterm::event::{self, KeyEventKind};
use env_logger::{Env, Target};
use ratatui::DefaultTerminal;
use std::fs::OpenOptions;

fn setup_logging() {
    let log_dir = dirs::data_local_dir().expect("failed to resolve local data directory");
    std::fs::create_dir_all(&log_dir).expect("failed to create log directory");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("logos.log"))
        .expect("failed to open logos.log");

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(Target::Pipe(Box::new(log_file)))
        .init();
}

fn app_loop(terminal: &mut DefaultTerminal) -> Result<()> {
    let keymap = KeyMap::default();
    terminal.clear()?;

    let mut state = AppStateEnum::Opening(StartupScreen::new());

    // Special startup logic.
    terminal.draw(|f| state.render(f).expect("render failed"))?;
    state = state.update(AppEvent::AppStart)?;

    loop {
        terminal.draw(|f| state.render(f).expect("render failed"))?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if let Some(action) = keymap.get(&key.code) {
                state = state.update(AppEvent::UserAction(action))?;
            }
        }

        if matches!(state, AppStateEnum::Exit) {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    setup_logging();
    color_eyre::install()?;
    ratatui::run(app_loop)?;
    Ok(())
}
