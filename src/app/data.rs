use crate::{bible::Bible, prelude::*};

/// The members of this struct should be cheap to copy, since the
/// persistent data will be passed between states.
pub struct PersistentAppData {
    pub bible: Arc<Bible>,
}

impl PersistentAppData {
    pub fn from_translation(translation: &str) -> Result<PersistentAppData> {
        Ok(PersistentAppData {
            bible: Arc::new(Bible::from_translation(translation)?),
        })
    }
}
