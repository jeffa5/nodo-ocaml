mod edit;
mod list;
mod new;
mod remove;

use std::io;

use crate::config::Config;
use crate::files::{NodoFile, ReadError, WriteError};
use crate::nodo::Nodo;

pub use edit::Edit;
pub use list::List;
pub use new::New;
pub use remove::Remove;

#[derive(Debug)]
pub struct CommandError(String);

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<io::Error> for CommandError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            _ => CommandError(format!("{}", err)),
        }
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

pub trait Command {
    fn exec<F: NodoFile>(config: Config, nodo: Nodo<F>) -> Result<(), CommandError>;
}
