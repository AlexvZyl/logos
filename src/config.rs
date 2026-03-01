use crate::prelude::*;
use std::sync::OnceLock;

type Translations = HashMap<&'static str, std::path::PathBuf>;

// TODO: Correctly locate assets.
static TRANSLATIONS: OnceLock<Translations> = OnceLock::new();

pub fn get_translations() -> &'static Translations {
    TRANSLATIONS
        .get_or_init(|| HashMap::from([("KJV", PathBuf::from("assets/eng-kjv.osis.xml.xz"))]))
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
