use log::*;
use std::path::Path;
use std::path::PathBuf;

use crate::cli::Target;
use crate::commands::{self, CommandError};
use crate::config::Config;

impl Target {
    pub fn build_path(&self, config: &Config, add_ext: bool) -> PathBuf {
        trace!("Building filepath");
        let mut pb = config.root_dir.join(&self.inner);
        if add_ext && pb.extension().is_none() && !self.is_empty() {
            pb.set_extension(&config.default_filetype);
        }
        debug!("Built filepath: {:?}", pb);
        pb
    }

    pub fn make_dirs(&self, config: &Config) -> std::io::Result<()> {
        let pb = config.root_dir.join(&self.inner);
        let projects = pb.parent().unwrap();
        info!("Creating the project dirs: {:?}", projects);
        std::fs::create_dir_all(projects)
    }
}

pub fn find_target(config: &Config, target: &Target) -> commands::Result<PathBuf> {
    let mut path = target.build_path(config, false);
    if path.exists() {
        return Ok(path);
    }
    if path.extension().is_none() {
        path.set_extension(&config.default_filetype);
        debug!("path: {:?}", &path);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(CommandError::TargetMissing(target.clone()))
}

pub fn is_hidden_dir(config: &Config, target_path: &Path, entry_path: &Path) -> bool {
    (entry_path.starts_with(&config.temp_dir) && !target_path.starts_with(&config.temp_dir))
        || (entry_path.starts_with(&config.archive_dir)
            && !target_path.starts_with(&config.archive_dir))
}
