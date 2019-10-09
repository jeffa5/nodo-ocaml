mod edit;
mod list;
mod new;
mod remove;

pub use edit::Edit;
pub use list::List;
pub use new::New;
pub use remove::Remove;

use crate::config::Config;
use crate::files::{NodoFile, ReadError, WriteError};
use crate::nodo::Nodo;

#[derive(Debug)]
pub enum CommandError {
    MissingFilename(&'static str),
    Io(std::io::Error),
    Read(ReadError),
    Write(WriteError),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommandError::MissingFilename(s) => write!(f, "Missing filename: {}", s),
            CommandError::Io(err) => write!(f, "{}", err),
            CommandError::Read(re) => write!(f, "{}", re),
            CommandError::Write(we) => write!(f, "{}", we),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<ReadError> for CommandError {
    fn from(err: ReadError) -> Self {
        Self::Read(err)
    }
}

impl From<WriteError> for CommandError {
    fn from(err: WriteError) -> Self {
        Self::Write(err)
    }
}

impl std::error::Error for CommandError {}

pub trait Command {
    fn exec<F: NodoFile>(config: Config, nodo: Nodo<F>) -> Result<(), CommandError>;
}
