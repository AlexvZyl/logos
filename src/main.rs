#![allow(dead_code)]

mod app;
mod bible;
mod components;
mod config;
mod error;
mod filesystem;
mod prelude;

use crate::app::events::{KeyMap, UserAction};
use crate::app::state::AppStateEnum;
use crate::app::state_startup_screen::StartupScreen;
use crate::prelude::*;
use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use env_logger::{Env, Target};
use ratatui::DefaultTerminal;
use std::fs::OpenOptions;
use std::time::Duration;

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
    // Special startup logic.
    let mut state = AppStateEnum::Opening(StartupScreen::new());
    terminal.draw(|f| {
        let _ = state.render(f).inspect_err(|e| error!("{e}"));
    })?;
    state = state.update(AppEvent::AppStart)?;

    let keymap = KeyMap::default();
    loop {
        // Wait for event.
        event::poll(Duration::MAX)?;

        // Process all events.  This prevents rendering from falling behind.
        while event::poll(Duration::from_millis(0))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if let Some(action) = keymap.get(&key.code, key.modifiers) {
                    trace!("Key: {}, Mod: {}", key.code, key.modifiers);
                    state = state.update(AppEvent::UserAction(action))?;
                }
            }

            if matches!(state, AppStateEnum::Exit) {
                return Ok(());
            }
        }

        terminal.draw(|f| {
            let _ = state.render(f).inspect_err(|e| error!("{e}"));
        })?;
    }
}

fn main() -> Result<()> {
    setup_logging();
    color_eyre::install()?;
    ratatui::run(app_loop)?;
    Ok(())
}
