use crate::bible::{Bible, BookId};
use crate::cache::{AppCache, ReaderCache};
use crate::components::book_column::Position;
use crate::prelude::*;

/// App data shared by all modes.  Owned by the app loop and lent to the current mode, so it
/// never has to be handed from one mode to the next.
pub struct Context {
    pub bible: Arc<Bible>,
    pub session: SessionData,
}

impl Context {
    pub fn load(translation: &str) -> Result<Context> {
        let bible = Arc::new(Bible::from_translation(translation)?);
        let session = SessionData::load(&bible);
        Ok(Context { bible, session })
    }
}

/// Where the user is in the bible.  Persisted between runs.
pub struct SessionData {
    pub book: BookId,
    pub position: Position,
    pub num_columns: usize,
}

impl SessionData {
    /// Never fails, a missing or broken cache, or a book not in `bible`, falls back to the start
    /// of the first book.
    pub fn load(bible: &Bible) -> SessionData {
        let reader = AppCache::load().reader;
        let (book, position) = match reader.book {
            Some(book) if bible.get_book(&book).is_ok() => {
                let position = Position {
                    chapter: reader.chapter,
                    verse: reader.verse,
                };
                (book, position)
            }
            _ => (bible.first_book(), Position::default()),
        };
        SessionData {
            book,
            position,
            num_columns: reader.num_columns,
        }
    }

    pub fn save(&self) {
        let cache = AppCache {
            reader: ReaderCache {
                book: Some(self.book.clone()),
                chapter: self.position.chapter,
                verse: self.position.verse,
                num_columns: self.num_columns,
            },
        };
        if let Err(e) = cache.save() {
            warn!("Failed to save cache: {e}");
        }
    }
}
