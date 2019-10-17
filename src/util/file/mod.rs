use log::*;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::config::Config;

pub fn create_file(config: &Config, target: &[String]) -> io::Result<fs::File> {
    let pb = build_path(config, target);
    info!("Creating file: {:?}", pb);
    fs::File::create(pb)
}

pub fn build_path(config: &Config, target: &[String]) -> PathBuf {
    trace!("Building filepath");
    let mut pb = PathBuf::from(&config.root_dir);
    target.iter().for_each(|project| pb.push(project));
    debug!("Built filepath: {:?}", pb);
    pb
}
