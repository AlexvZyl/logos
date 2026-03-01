use crossterm::event::KeyCode;
use std::collections::HashMap;

pub enum AppEvent {
    /// Special event fired at start of app.
    AppStart,
    /// Action performed/requested by the user.
    UserAction(UserAction),
    Focus,
    Defocus,
}

#[derive(Clone, Copy)]
pub enum UserAction {
    Quit,
    MoveUp,
    MoveDown,
    IncrementWindow,
    DecrementWindow,
}

pub struct KeyMap(HashMap<KeyCode, UserAction>);

impl KeyMap {
    // TODO: This should go to a config file.
    pub fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(KeyCode::Char('q'), UserAction::Quit);
        map.insert(KeyCode::Char('j'), UserAction::MoveDown);
        map.insert(KeyCode::Char('k'), UserAction::MoveUp);
        map.insert(KeyCode::Tab, UserAction::IncrementWindow);
        map.insert(KeyCode::BackTab, UserAction::DecrementWindow);
        KeyMap(map)
    }

    pub fn get(&self, key: &KeyCode) -> Option<UserAction> {
        self.0.get(key).copied()
    }
}
