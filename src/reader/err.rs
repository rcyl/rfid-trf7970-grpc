use std::fmt;
use std::fmt::Debug;

use crate::serial::err::SerialError;

#[derive(Debug)]
pub enum ReaderError {
    SerialError(SerialError),
    //returns the whatever is read when no matching targets
    NoMatchingTargets(String),
    InvalidRegex(String),
    BlockIdxTooLarge(u32),
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReaderError::SerialError(ref e) => std::fmt::Display::fmt(&e, f),
            ReaderError::NoMatchingTargets(ref e) => {
                let mut s = "Targets not found, recv: ".to_owned();
                s.push_str(e);
                write!(f, "{}", s.to_string())
            }
            ReaderError::InvalidRegex(ref e) => {
                let mut s = "Invalid regex: ".to_owned();
                s.push_str(e);
                write!(f, "{}", s.to_string())
            }
            ReaderError::BlockIdxTooLarge(e) => {
                let s = format!("Block index is too large: {}", e);
                write!(f, "{}", s.to_string())
            }
        }
    }
}

impl From<SerialError> for ReaderError {
    fn from(err: SerialError) -> ReaderError {
        ReaderError::SerialError(err)
    }
}
