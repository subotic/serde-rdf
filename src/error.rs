//! The `Error` and `Result` types used by this crate.

use std::fmt::{Display, Formatter};
use std::io;
use std::str::Utf8Error;

use serde::{de, ser};

/// The result type used by this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The error type used by this crate.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Represents a generic error message.
    Message(String),
    /// Represents an error that resulted from invalid UTF8 input.
    Utf8(Utf8Error),
    /// Represents generic IO errors.
    Io(io::Error),
    /// Represents an error during serialization.
    CannotSerializePrimitive(&'static str),
}

impl Error {
    pub(crate) fn new<T>(msg: T) -> Error
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{msg}"),
            Error::Utf8(err) => write!(f, "{err}"),
            Error::Io(err) => write!(f, "{err}"),
            Error::CannotSerializePrimitive(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new(msg)
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::new(msg)
    }
}
