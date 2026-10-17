use ratatui::style::{Color, Modifier, Style};

pub const TEXT: Style = Style::new().fg(Color::White);
pub const MUTED: Style = Style::new().fg(Color::DarkGray);
pub const TITLE: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
pub const BORDER: Style = Style::new();
pub const BORDER_FOCUSED: Style = Style::new().fg(Color::Blue);
pub const SELECTED: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);

pub const CHAPTER_HEADING: Style = Style::new().fg(Color::Blue).add_modifier(Modifier::ITALIC);
pub const VERSE_NUMBER: Style = Style::new().fg(Color::DarkGray);

pub const KEY_HINT: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
pub const KEY_HINT_DANGER: Style = Style::new().fg(Color::Red).add_modifier(Modifier::BOLD);

pub const LOGO: Style = Style::new().fg(Color::Yellow);
pub const FOOTER: Style = Style::new().bg(Color::Black);
pub const FOOTER_APP_NAME: Style = Style::new().fg(Color::White).add_modifier(Modifier::BOLD);
pub const FOOTER_VERSION: Style = Style::new().fg(Color::DarkGray).add_modifier(Modifier::BOLD);
pub const FOOTER_KEYMAPS: Style = Style::new().fg(Color::Red).add_modifier(Modifier::BOLD);

pub fn border(focused: bool) -> Style {
    if focused { BORDER_FOCUSED } else { BORDER }
}
