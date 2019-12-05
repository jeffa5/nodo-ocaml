use git2::Repository;
use log::*;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
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

    pub fn make_dirs(&self, config: &Config) -> io::Result<()> {
        let pb = config.root_dir.join(&self.inner);
        let projects = pb.parent().unwrap();
        info!("Creating the project dirs: {:?}", projects);
        fs::create_dir_all(projects)
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

pub fn local_file(config: &Config) -> PathBuf {
    let mut base = env::current_dir().unwrap();
    // if in git repo then add to exclude
    if let Ok(repo) = Repository::discover(&base) {
        let git_base = repo.path().to_path_buf();

        let exclude_file = git_base.join("info/exclude");

        let exclude_contents = fs::read_to_string(&exclude_file).unwrap();
        if !exclude_contents.contains(".nodo") {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(&exclude_file)
                .unwrap();

            if let Err(e) = writeln!(file, ".nodo.*") {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
        base = git_base
            .parent()
            .expect("Failed to find parent of .git folder")
            .to_path_buf()
    }
    let mut nodo_file = base.join(".nodo");
    nodo_file.set_extension(&config.default_filetype);
    nodo_file
}
