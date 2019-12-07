use std::io;

use crate::cli::Target;
use crate::files::{ReadError, WriteError};

#[derive(Debug, PartialEq)]
pub enum CommandError {
    /// No target was provided
    NoTarget,
    /// Target provided but doesn't exist in filesystem
    TargetMissing(Target),
    /// Generic error
    Str(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommandError::NoTarget => write!(f, "Please provide a target"),
            CommandError::TargetMissing(target) => {
                if target.inner == "" {
                    write!(f, "Couldn't find local file")
                } else {
                    write!(f, "Couldn't find target: '{}'", target)
                }
            }
            CommandError::Str(s) => write!(f, "{}", s),
        }
    }
}

impl From<io::Error> for CommandError {
    fn from(err: io::Error) -> Self {
        CommandError::Str(format!("An unhandled io error was encountered: {:?}", err))
    }
}

impl From<&str> for CommandError {
    fn from(err: &str) -> Self {
        CommandError::Str(err.to_string())
    }
}

impl From<ReadError> for CommandError {
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

impl From<WriteError> for CommandError {
    fn from(err: WriteError) -> Self {
        match err {
            WriteError::Io(ioerr) => ioerr.into(),
        }
    }
}

impl std::error::Error for CommandError {}
