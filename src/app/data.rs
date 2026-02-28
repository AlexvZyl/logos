use crate::{bible::Bible, prelude::*};

/// All of the members of this type has to be cheap to move, since it
/// will move between states.
pub struct AppData {
    pub bible: Box<Bible>,
}

impl AppData {
    pub fn from_translation(translation: &str) -> Result<AppData> {
        Ok(AppData {
            bible: Box::new(Bible::from_translation(translation)?),
        })
    }
}
