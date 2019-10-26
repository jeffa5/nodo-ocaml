use log::*;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::config::Config;

pub fn create_file(path: &std::path::Path) -> io::Result<fs::File> {
    info!("Creating file: {:?}", path);
    fs::File::create(path)
}

pub fn build_path(config: &Config, target: &str, add_ext: bool) -> PathBuf {
    trace!("Building filepath");
    let mut pb = config.root_dir.join(target);
    if add_ext && pb.extension().is_none() && !target.is_empty() {
        pb.set_extension(&config.default_filetype);
    }
    debug!("Built filepath: {:?}", pb);
    pb
}
