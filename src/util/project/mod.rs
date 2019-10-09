use crate::config::Config;
use log::*;
use std::path::PathBuf;

use crate::files::NodoFile;
use crate::nodo::Nodo;

pub fn make_dirs<F: NodoFile>(config: &Config, nodo: &Nodo<F>) -> std::io::Result<()> {
    let mut pb = PathBuf::from(&config.root_dir);
    nodo.metadata()
        .projects()
        .iter()
        .for_each(|project| pb.push(project));
    info!("Creating the project dirs: {:?}", pb);
    std::fs::create_dir_all(pb)
}
