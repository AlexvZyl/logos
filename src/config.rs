use crate::prelude::*;
use std::{sync::OnceLock, time::Duration};

/// xz compressed OSIS files, built into the binary.
type Translations = HashMap<&'static str, &'static [u8]>;

static TRANSLATIONS: OnceLock<Translations> = OnceLock::new();

pub fn get_translations() -> &'static Translations {
    TRANSLATIONS.get_or_init(|| {
        HashMap::from([("KJV", include_bytes!("../assets/eng-kjv.osis.xml.xz").as_slice())])
    })
}

pub const VERSION: &str = env!("GIT_HASH");

// TODO: Get from screen rate or config file.
pub const TARGET_FRAMERATE: f64 = 120.0;
pub const TARGET_FRAMETIME: Duration = Duration::from_micros((1000000.0 / TARGET_FRAMERATE) as u64);
