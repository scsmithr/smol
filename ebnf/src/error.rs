use std::error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    ParseError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError(ref s) => write!(f, "failed to parse: {}", s),
        }
    }
}

impl error::Error for Error {}

impl From<nom::Err<nom::error::Error<&str>>> for Error {
    fn from(err: nom::Err<nom::error::Error<&str>>) -> Error {
        Error::ParseError(format!("{:?}", err))
    }
}
