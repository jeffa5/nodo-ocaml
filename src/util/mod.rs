use log::*;
use std::path::PathBuf;

use crate::cli::Target;
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
