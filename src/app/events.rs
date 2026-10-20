use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

pub enum AppEvent {
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
    OpenReader,
    IncrementColumns,
    DecrementColumns,
    HalfPageDown,
    HalfPageUp,
    NextPage,
    PrevPage,
    ToggleHelp,
}

impl UserAction {
    pub fn description(&self) -> &'static str {
        match self {
            UserAction::Quit => "Quit",
            UserAction::MoveUp => "Move up",
            UserAction::MoveDown => "Move down",
            UserAction::IncrementWindow => "Next window",
            UserAction::DecrementWindow => "Previous window",
            UserAction::JumpToWindow(_) => "Jump to window",
            UserAction::OpenReader => "Open reader",
            UserAction::IncrementColumns => "Add column",
            UserAction::DecrementColumns => "Remove column",
            UserAction::HalfPageDown => "Half page down",
            UserAction::HalfPageUp => "Half page up",
            UserAction::NextPage => "Next page",
            UserAction::PrevPage => "Previous page",
            UserAction::ToggleHelp => "Toggle keybinds",
        }
    }
}

impl UserAction {
    /// Same action, ignoring data such as the `JumpToWindow` index.
    pub fn same(self, other: UserAction) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&other)
    }
}

/// Available actions, grouped into help sections in display order.
pub type ActionSections = &'static [&'static [UserAction]];

pub const GLOBAL: &[UserAction] = &[UserAction::ToggleHelp, UserAction::Quit];
/// A single `JumpToWindow` covers all of its indices.
pub const WINDOWS: &[UserAction] = &[
    UserAction::IncrementWindow,
    UserAction::DecrementWindow,
    UserAction::JumpToWindow(0),
];
pub const NAVIGATION: &[UserAction] = &[
    UserAction::MoveDown,
    UserAction::MoveUp,
    UserAction::HalfPageDown,
    UserAction::HalfPageUp,
    UserAction::NextPage,
    UserAction::PrevPage,
];
pub const COLUMNS: &[UserAction] = &[UserAction::IncrementColumns, UserAction::DecrementColumns];

/// Mappings of keys -> actions.
///
/// TODO: This should go to a config file.
pub struct KeyMap(HashMap<(KeyCode, KeyModifiers), UserAction>);

impl KeyMap {
    pub fn default() -> Self {
        let mut map = HashMap::new();
        let none = KeyModifiers::NONE;
        map.insert((KeyCode::Char('q'), none), UserAction::Quit);
        map.insert((KeyCode::Char('c'), KeyModifiers::CONTROL), UserAction::Quit);
        map.insert((KeyCode::Char('r'), none), UserAction::OpenReader);
        map.insert((KeyCode::Char('j'), none), UserAction::MoveDown);
        map.insert((KeyCode::Char('k'), none), UserAction::MoveUp);
        map.insert((KeyCode::Char('+'), none), UserAction::IncrementColumns);
        map.insert((KeyCode::Char('+'), KeyModifiers::SHIFT), UserAction::IncrementColumns);
        map.insert((KeyCode::Char('-'), none), UserAction::DecrementColumns);
        map.insert((KeyCode::Char('d'), KeyModifiers::CONTROL), UserAction::HalfPageDown);
        map.insert((KeyCode::Char('u'), KeyModifiers::CONTROL), UserAction::HalfPageUp);
        map.insert((KeyCode::Char('n'), none), UserAction::NextPage);
        map.insert((KeyCode::Char('p'), none), UserAction::PrevPage);
        map.insert((KeyCode::Char('?'), none), UserAction::ToggleHelp);
        map.insert((KeyCode::Char('?'), KeyModifiers::SHIFT), UserAction::ToggleHelp);
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

    /// Returns `(keys, description)` per section, with all keys for the same action
    /// grouped.  Plain keys are listed before modified ones.
    pub fn help_entries(&self, sections: ActionSections) -> Vec<Vec<(String, &'static str)>> {
        let mut bindings: Vec<_> = self.0.iter().collect();
        bindings.sort_by_key(|&(&(code, mods), _)| (mods.bits(), format_key(&code, mods)));

        let keys_for = |action: &UserAction| -> String {
            let mut keys: Vec<String> = Vec::new();
            for &(&(code, modifiers), a) in &bindings {
                let key = format_key(&code, modifiers);
                // Shifted characters format the same as unshifted ones.
                if a.same(*action) && !keys.contains(&key) {
                    keys.push(key);
                }
            }
            keys.join(", ")
        };

        sections
            .iter()
            .map(|actions| {
                actions
                    .iter()
                    .map(|a| (keys_for(a), a.description()))
                    .filter(|(keys, _)| !keys.is_empty())
                    .collect()
            })
            .collect()
    }
}

/// Shift is implied by the character itself, so it is not shown.
fn format_key(code: &KeyCode, modifiers: KeyModifiers) -> String {
    let code = match code {
        KeyCode::BackTab => "Shift+Tab".to_string(),
        c => c.to_string(),
    };
    let ctrl = if modifiers.contains(KeyModifiers::CONTROL) { "Ctrl+" } else { "" };
    let alt = if modifiers.contains(KeyModifiers::ALT) { "Alt+" } else { "" };
    format!("{ctrl}{alt}{code}")
}
