use log::*;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::config::Config;
use crate::files::NodoFile;
use crate::nodo::Nodo;

pub fn create_file<F: NodoFile>(config: &Config, nodo: &Nodo<F>) -> io::Result<fs::File> {
    let pb = build_path(config, nodo);
    info!("Creating file: {:?}", pb);
    fs::File::create(pb)
}

pub fn remove_file<F: NodoFile>(config: &Config, nodo: &Nodo<F>) -> io::Result<()> {
    let pb = build_path(config, nodo);
    info!("Removing file: {:?}", pb);
    // don't care if it fails
    fs::remove_file(pb)
}

pub fn build_path<F: NodoFile>(config: &Config, nodo: &Nodo<F>) -> PathBuf {
    trace!("Building filepath");
    let mut pb = PathBuf::from(&config.root_dir);
    nodo.metadata()
        .projects()
        .iter()
        .for_each(|project| pb.push(project));
    pb.push(nodo.metadata().target());
    debug!("Built filepath: {:?}", pb);
    pb
}
