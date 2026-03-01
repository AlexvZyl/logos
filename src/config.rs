use crate::prelude::*;
use std::{sync::OnceLock, time::Duration};

type Translations = HashMap<&'static str, std::path::PathBuf>;

// TODO: Correctly locate assets.
static TRANSLATIONS: OnceLock<Translations> = OnceLock::new();

pub fn get_translations() -> &'static Translations {
    TRANSLATIONS
        .get_or_init(|| HashMap::from([("KJV", PathBuf::from("assets/eng-kjv.osis.xml.xz"))]))
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: Get from screen rate or config file.
pub const TARGET_FRAMERATE: f64 = 120.0;
pub const TARGET_FRAMETIME: Duration = Duration::from_millis((1000.0 / TARGET_FRAMERATE) as u64);
