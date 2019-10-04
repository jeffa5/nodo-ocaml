/// The files module provides readers and writers for different file types
use crate::nodo::Nodo;

#[derive(Debug)]
pub enum ReadError {
    InvalidElement(String),
}

#[derive(Debug)]
pub enum WriteError {
    IOError(std::io::Error),
}

/// A reader for nodos
pub trait NodoReader {
    fn read<R: std::io::Read>(&mut self, r: R) -> Result<(), ReadError>;
}

/// A writer for nodos
pub trait NodoWriter {
    fn write<W: std::io::Write>(&mut self, w: W) -> Result<(), WriteError>;
}

/// A provider of nodos
pub trait NodoProvider {
    fn nodo(&self) -> &Nodo;
}
