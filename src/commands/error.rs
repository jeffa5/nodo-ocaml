use std::io;

use crate::cli::Target;
use crate::files::{ReadError, WriteError};

#[derive(Debug, PartialEq)]
pub enum CommandError<'a> {
    /// No target was provided
    NoTarget,
    /// Target provided but doesn't exist in filesystem
    TargetMissing(&'a Target),
    /// Generic error
    Str(String),
}

impl<'a> std::fmt::Display for CommandError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommandError::NoTarget => write!(f, "Please provide a target"),
            CommandError::TargetMissing(target) => write!(f, "Couldn't find target: '{}'", target),
            CommandError::Str(s) => write!(f, "{}", s),
        }
    }
}

impl<'a> From<io::Error> for CommandError<'a> {
    fn from(err: io::Error) -> Self {
        CommandError::Str(format!("An unhandled io error was encountered: {:?}", err))
    }
}

impl<'a> From<&str> for CommandError<'a> {
    fn from(err: &str) -> Self {
        CommandError::Str(err.to_string())
    }
}

impl<'a> From<ReadError> for CommandError<'a> {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::Io(ioerr) => ioerr.into(),
            ReadError::InvalidElement(s) => CommandError::Str(format!(
                "Encountered invalid element when reading nodo: {}",
                s
            )),
            ReadError::Str(s) => CommandError::Str(s),
        }
    }
}

impl<'a> From<WriteError> for CommandError<'a> {
    fn from(err: WriteError) -> Self {
        match err {
            WriteError::Io(ioerr) => ioerr.into(),
        }
    }
}

impl<'a> From<walkdir::Error> for CommandError<'a> {
    fn from(err: walkdir::Error) -> Self {
        CommandError::Str(format!("{}", err))
    }
}

impl<'a> std::error::Error for CommandError<'a> {}
