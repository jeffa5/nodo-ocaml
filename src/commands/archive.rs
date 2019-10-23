use log::*;
use std::fs;

use crate::cli::Archive;
use crate::commands::CommandError;
use crate::config::Config;
use crate::util::file;

impl Archive {
    /// Archive a nodo or project tree
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        trace!("Archiving {:?}", self.target);
        if self.target.is_empty() {
            return Err(CommandError::NoTarget);
        }
        let mut path = file::build_path(&config, &self.target, false);
        if !path.exists() && path.extension().is_none() {
            path.set_extension(config.default_filetype);
        }
        let move_target = config.archive_dir.join(&self.target.inner);
        debug!("Moving {:?} to {:?}", path, move_target);
        if path.is_file() {
            fs::rename(path, move_target)?;
        } else if path.is_dir() {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::rename(path, move_target)?;
        } else {
            return Err(CommandError::TargetMissing(&self.target));
        }
        println!("Archive successful");
        Ok(())
    }
}
