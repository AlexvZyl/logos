use crate::bible::Bible;
use ratatui::{DefaultTerminal, Frame};
use std::path::Path;

fn render(frame: &mut Frame) {
    let bible = Bible::from_file(Path::new("assets/eng-kjv.osis.xml.xz")).unwrap();
    for part in bible.get_verse_iter("Ephesians", 1, 1).unwrap() {
        frame.render_widget(part, frame.area());
    }
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}
