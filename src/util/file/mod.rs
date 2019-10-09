use log::*;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::config::Config;
use crate::files::NodoFile;
use crate::nodo::Nodo;

pub fn create_file<F: NodoFile>(config: &Config, nodo: &Nodo<F>) -> io::Result<fs::File> {
    let pb = build_filepath(config, nodo);
    info!("Creating file: {:?}", pb);
    fs::File::create(pb)
}

pub fn remove_file<F: NodoFile>(config: &Config, nodo: &Nodo<F>) -> io::Result<()> {
    let pb = build_filepath(config, nodo);
    info!("Removing file: {:?}", pb);
    // don't care if it fails
    fs::remove_file(pb)
}

pub fn build_dirpath<F: NodoFile>(config: &Config, nodo: &Nodo<F>) -> PathBuf {
    trace!("Building dirpath");
    let mut pb = PathBuf::from(&config.root_dir);
    nodo.metadata()
        .projects()
        .iter()
        .for_each(|project| pb.push(project));
    debug!("Built dirpath: {:?}", pb);
    pb
}

pub fn build_filepath<F: NodoFile>(config: &Config, nodo: &Nodo<F>) -> PathBuf {
    trace!("Building filepath");
    let mut pb = PathBuf::from(&config.root_dir);
    nodo.metadata()
        .projects()
        .iter()
        .for_each(|project| pb.push(project));
    pb.push(nodo.metadata().filename());
    pb.set_extension(F::EXTENSION);
    debug!("Built filepath: {:?}", pb);
    pb
}

pub fn list_dir<F: NodoFile>(config: &Config, nodo: &Nodo<F>) -> fs::ReadDir {
    let pb = build_dirpath(config, nodo);
    info!("Listing files of: {:?}", pb);
    fs::read_dir(pb).expect("Failed to read directory contents")
}
