#![allow(dead_code)]

mod bible;
mod error;
mod filesystem;
mod prelude;
mod tui;

use crate::{prelude::*, tui::app};
use env_logger::{Env, Target};
use std::fs::OpenOptions;

// TODO: Improve error handling.
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

fn main() -> Result<()> {
    setup_logging();
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}
