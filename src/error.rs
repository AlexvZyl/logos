use std::string::FromUtf8Error;

use color_eyre::eyre;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid file for bible")]
    InvalidBibleFile,
    #[error("Could not create bible index")]
    BibleIndex(String),
    #[error("No matching book")]
    BookNotFound(String),
    #[error("No matching chapter")]
    ChapterNotFound(String, usize),
    #[error("No matching chapter")]
    VerseNotFound(String, usize, usize),
    #[error("Translation not supported")]
    UnsupprtedTranslation(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ParseError(#[from] FromUtf8Error),

    #[error(transparent)]
    Eyre(#[from] eyre::Report),
}

pub type Result<T> = std::result::Result<T, Error>;
