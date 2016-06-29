use std::fmt;
use std::error::Error as StdError;
use serde::de::Error as SerdeError;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    MissingValue(&'static str),
    Custom(String)
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::MissingValue(_) => "missing value",
            Error::Custom(_) => "custom error"
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::MissingValue(field) => write!(fmt, "missing value for field {}", field),
            Error::Custom(ref msg) => write!(fmt, "{}", msg),
        }
    }
}

impl SerdeError for Error {
    fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Custom(msg.into())
    }

    fn missing_field(field: &'static str) -> Error {
        Error::MissingValue(field)
    }

    fn end_of_stream() -> Error {
        println!("end of stream");
        Error::MissingValue("fixme")
    }
}
