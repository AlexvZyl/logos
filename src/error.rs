use std::string::FromUtf8Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ParseError(#[from] FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;
