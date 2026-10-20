use crate::bible::BookId;
use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// App state persisted between runs.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppCache {
    pub reader: ReaderCache,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ReaderCache {
    pub book: Option<BookId>,
    pub chapter: usize,
    pub verse: usize,
    pub num_columns: usize,
}

impl Default for ReaderCache {
    fn default() -> Self {
        Self {
            book: None,
            chapter: 0,
            verse: 0,
            num_columns: 2,
        }
    }
}

impl AppCache {
    fn path() -> Result<PathBuf> {
        dirs::cache_dir()
            .map(|d| d.join("logos").join("cache.toml"))
            .ok_or(Error::NoCacheDir)
    }

    /// Never fails, a missing or broken cache falls back to the default.
    pub fn load() -> Self {
        let read = || -> Result<Self> {
            let path = Self::path()?;
            if !path.exists() {
                return Ok(Self::default());
            }
            Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
        };
        read().unwrap_or_else(|e| {
            warn!("Failed to load cache: {e}");
            Self::default()
        })
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }
}
