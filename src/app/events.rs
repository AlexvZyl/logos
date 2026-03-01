use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

pub enum AppEvent {
    /// Special event fired at start of app.
    AppStart,
    /// Action performed/requested by the user.
    UserAction(UserAction),
    /// Window focus, specific to components.
    Focus,
    /// Window defocus, specific to components.
    Defocus,
}

/// Actions that can be performed by the user.  They all should have key mappings.
#[derive(Clone, Copy)]
pub enum UserAction {
    Quit,
    MoveUp,
    MoveDown,
    IncrementWindow,
    DecrementWindow,
    /// `usize`: The index of the window.
    JumpToWindow(usize),
}

/// Mappings of keys -> actions.
///
/// TODO: This should go to a config file.
pub struct KeyMap(HashMap<(KeyCode, KeyModifiers), UserAction>);

impl KeyMap {
    pub fn default() -> Self {
        let mut map = HashMap::new();
        let none = KeyModifiers::NONE;
        map.insert((KeyCode::Char('q'), none), UserAction::Quit);
        map.insert((KeyCode::Char('j'), none), UserAction::MoveDown);
        map.insert((KeyCode::Char('k'), none), UserAction::MoveUp);
        map.insert((KeyCode::Tab, none), UserAction::IncrementWindow);
        map.insert(
            (KeyCode::BackTab, KeyModifiers::SHIFT),
            UserAction::DecrementWindow,
        );
        map.insert((KeyCode::Char('1'), none), UserAction::JumpToWindow(0));
        map.insert((KeyCode::Char('2'), none), UserAction::JumpToWindow(1));
        map.insert((KeyCode::Char('3'), none), UserAction::JumpToWindow(2));
        map.insert((KeyCode::Char('4'), none), UserAction::JumpToWindow(3));
        map.insert((KeyCode::Char('5'), none), UserAction::JumpToWindow(4));
        KeyMap(map)
    }

    pub fn get(&self, key: &KeyCode, modifiers: KeyModifiers) -> Option<UserAction> {
        self.0.get(&(*key, modifiers)).copied()
    }
}
