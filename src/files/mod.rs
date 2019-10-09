/// The files module provides readers and writers for different file types
pub mod markdown;

use crate::nodo::Nodo;

#[derive(Debug)]
pub enum ReadError {
    InvalidElement(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for ReadError {
    fn from(err: std::io::Error) -> ReadError {
        ReadError::Io(err)
    }
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ReadError::InvalidElement(s) => write!(f, "{}", s),
            ReadError::Io(ioerr) => write!(f, "{}", ioerr),
        }
    }
}

impl std::error::Error for ReadError {}

#[derive(Debug)]
pub enum WriteError {
    Io(std::io::Error),
}

impl From<std::io::Error> for WriteError {
    fn from(err: std::io::Error) -> WriteError {
        WriteError::Io(err)
    }
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WriteError::Io(ioerr) => write!(f, "{}", ioerr),
        }
    }
}

impl std::error::Error for WriteError {}

pub trait NodoFile: std::fmt::Debug {
    const EXTENSION: &'static str;
    fn read<R>(nodo: Nodo<Self>, r: &mut R) -> Result<Nodo<Self>, ReadError>
    where
        Self: Sized + NodoFile,
        R: std::io::Read;
    fn write<W>(nodo: &Nodo<Self>, w: &mut W) -> Result<(), WriteError>
    where
        W: std::io::Write,
        Self: Sized;
}

pub fn get_nodo(ft: &str) -> Nodo<impl NodoFile> {
    match ft {
        markdown::Markdown::EXTENSION => Nodo::<markdown::Markdown>::new(),
        _ => panic!("Couldn't get file handler"),
    }
}
