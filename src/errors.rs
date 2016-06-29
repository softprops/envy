
use std::fmt;
use std::error;
use serde::de;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    MissingValue,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::MissingValue => "missing value",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::MissingValue => write!(fmt, "missing value"),
        }
    }
}

impl de::Error for Error {
    fn custom<T: Into<String>>(msg: T) -> Error {
        println!("custom err: {}", msg.into());
        Error::MissingValue
    }

    fn end_of_stream() -> Error {
        println!("end of stream");
        Error::MissingValue
    }
}
