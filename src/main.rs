mod app;
mod bible;
mod cache;
mod chars;
mod components;
mod config;
mod error;
mod filesystem;
mod prelude;
mod theme;

use crate::app::events::{KeyMap, UserAction};
use crate::components::Component;
use crate::components::confirm::Confirm;
use crate::components::help_overlay::HelpOverlay;
use crate::components::prompt::{Prompt, PromptKey};
use crate::app::command;
use crate::app::context::Context;
use crate::app::modes::Mode;
use crate::app::modes::dashboard::Dashboard;
use crate::components::splash_screen::SplashScreen;
use crate::prelude::*;
use crossterm::event::{self, KeyEventKind};
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
    info!("Target framerate: {TARGET_FRAMERATE}fps");
    info!("Target frametime: {TARGET_FRAMETIME:?}");

    // Show the splash screen while the bible loads.
    terminal.draw(|f| {
        let _ = SplashScreen
            .render(f.area(), f.buffer_mut())
            .inspect_err(|e| error!("{e}"));
    })?;
    let mut ctx = Context::load("KJV")?;
    let mut mode = Mode::Dashboard(Dashboard::new());

    let keymap = KeyMap::default();
    let mut help: Option<HelpOverlay> = None;
    let mut prompt: Option<Prompt> = None;
    let mut confirm_quit: Option<Confirm> = None;
    loop {
        // Wait indefinitely for event.
        // TODO: I really want some kind of tick mechanism.
        event::poll(Duration::MAX)?;

        loop {
            // Only stop processing when no events are left.
            if !event::poll(Duration::ZERO)? {
                break;
            }

            if let event::Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Any key closes the help overlay.
                if help.take().is_some() {
                    continue;
                }

                if let Some(confirm) = &confirm_quit {
                    match confirm.key(key) {
                        Some(true) => {
                            confirm_quit = None;
                            mode = mode.update(&mut ctx, &AppEvent::UserAction(UserAction::Quit))?;
                        }
                        Some(false) => confirm_quit = None,
                        None => {}
                    }
                } else if let Some(open) = &mut prompt {
                    // The prompt takes all keys while open, bypassing the keymap.
                    match open.key(key) {
                        PromptKey::Edit => {
                            open.set_suggestion(command::complete(open.input(), &ctx.bible));
                        }
                        PromptKey::Cancel => prompt = None,
                        PromptKey::Submit(input) => match command::parse(&input, &ctx.bible) {
                            Ok(None) => prompt = None,
                            Ok(Some(event)) => {
                                if let AppEvent::UserAction(action) = event
                                    && !available(&mode, action)
                                {
                                    open.set_error(format!("Not available: {input}"));
                                } else {
                                    prompt = None;
                                    mode = mode.update(&mut ctx, &event)?;
                                }
                            }
                            Err(error) => open.set_error(error),
                        },
                    }
                } else if let Some(action) = keymap.get(&key.code, key.modifiers) {
                    trace!("Key: {}, Mod: {}", key.code, key.modifiers);
                    if !available(&mode, action) {
                        continue;
                    }
                    match action {
                        UserAction::ToggleHelp => {
                            help = Some(HelpOverlay::new(&keymap, mode.actions()));
                        }
                        UserAction::OpenPrompt => prompt = Some(Prompt::new()),
                        // Quitting from the prompt with `:q` does not ask.
                        UserAction::Quit if mode.confirm_quit() => {
                            confirm_quit = Some(Confirm::new("Quit logos?"));
                        }
                        _ => mode = mode.update(&mut ctx, &AppEvent::UserAction(action))?,
                    }
                }
            }

            if matches!(mode, Mode::Exit) {
                info!("Exiting");
                ctx.session.save();
                return Ok(());
            }
        }

        // TODO: I want rendering statistics.
        terminal.draw(|f| {
            let _ = mode.render(&ctx, f).inspect_err(|e| error!("{e}"));
            if let Some(help) = &mut help {
                let _ = help.render(f.area(), f.buffer_mut()).inspect_err(|e| error!("{e}"));
            }
            if let Some(confirm) = &mut confirm_quit {
                let _ = confirm.render(f.area(), f.buffer_mut()).inspect_err(|e| error!("{e}"));
            }
            if let Some(prompt) = &mut prompt {
                let area = Prompt::area(f.area());
                let _ = prompt.render(area, f.buffer_mut()).inspect_err(|e| error!("{e}"));
            }
        })?;
    }
}

fn available(mode: &Mode, action: UserAction) -> bool {
    mode.actions().iter().flat_map(|s| s.iter()).any(|a| a.same(action))
}

fn main() -> Result<()> {
    setup_logging();
    color_eyre::install()?;
    ratatui::run(app_loop)?;
    Ok(())
}
