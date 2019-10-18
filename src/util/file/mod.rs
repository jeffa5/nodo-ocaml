use log::*;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::config::Config;

pub fn create_file(path: &std::path::Path) -> io::Result<fs::File> {
    info!("Creating file: {:?}", path);
    fs::File::create(path)
}

pub fn build_path(config: &Config, target: &[String]) -> PathBuf {
    trace!("Building filepath");
    let mut pb = PathBuf::from(&config.root_dir);
    target.iter().for_each(|project| pb.push(project));
    if pb.extension().is_none() {
        pb.set_extension(config.default_filetype);
    }
    debug!("Built filepath: {:?}", pb);
    pb
}
