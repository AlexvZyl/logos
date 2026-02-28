use std::string::FromUtf8Error;

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

    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ParseError(#[from] FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;
