mod edit;
mod format;
mod list;
mod new;
mod overview;
mod remove;

use std::io;

use crate::files::{ReadError, WriteError};

#[derive(Debug, PartialEq)]
pub struct CommandError(String);

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<io::Error> for CommandError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => CommandError("Couldn't find target".to_string()),
            _ => CommandError(format!("{:?}", err)),
        }
    }
}

impl From<&str> for CommandError {
    fn from(err: &str) -> Self {
        CommandError(err.to_string())
    }
}

impl From<ReadError> for CommandError {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::Io(ioerr) => ioerr.into(),
            ReadError::InvalidElement(s) => CommandError(format!(
                "Encountered invalid element when reading nodo: {}",
                s
            )),
            ReadError::Str(s) => CommandError(s),
        }
    }
}

impl From<WriteError> for CommandError {
    fn from(err: WriteError) -> Self {
        match err {
            WriteError::Io(ioerr) => ioerr.into(),
        }
    }
}

impl From<walkdir::Error> for CommandError {
    fn from(err: walkdir::Error) -> Self {
        CommandError(format!("{}", err))
    }
}

impl std::error::Error for CommandError {}
