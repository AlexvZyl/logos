use std::ops::Deref;

use crate::{bible::Bible, prelude::*};

// TODO: Revisit the name of this... Not sure if this makes sense.
// That or this needs more state.
pub struct AppDataInner {
    pub bible: Bible,
}

/// Using an `Arc` here as this can be required to be shared and has to be cheap
/// to copy/clone.
pub struct AppData(Arc<AppDataInner>);

impl AppData {
    pub fn from_translation(translation: &str) -> Result<AppData> {
        Ok(AppData(Arc::new(AppDataInner {
            bible: Bible::from_translation(translation)?,
        })))
    }
}

impl Deref for AppData {
    type Target = AppDataInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
