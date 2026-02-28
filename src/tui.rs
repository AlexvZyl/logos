use crate::bible::Bible;
use crate::prelude::*;
use ratatui::{DefaultTerminal, Frame};

fn render(frame: &mut Frame, bible: &Bible) {
    for part in bible.get_verse_iter("Ephesians", 1, 1).unwrap() {
        frame.render_widget(part, frame.area());
    }
}

pub fn app(terminal: &mut DefaultTerminal) -> Result<()> {
    let bible = Bible::from_translation("KJV")?;

    loop {
        terminal.draw(|f| render(f, &bible))?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}
