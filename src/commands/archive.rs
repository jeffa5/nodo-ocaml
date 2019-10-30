use log::*;
use std::fs;

use crate::cli::Archive;
use crate::commands::{self, CommandError};
use crate::config::Config;

impl Archive {
    /// Archive a nodo or project tree
    pub fn exec(&self, config: Config) -> commands::Result<()> {
        trace!("Archiving {:?}", self.target);
        if self.target.is_empty() {
            return Err(CommandError::NoTarget);
        }
        let mut path = self.target.build_path(&config, false);
        if !path.exists() && path.extension().is_none() {
            path.set_extension(&config.default_filetype);
        }
        let mut move_target = config.archive_dir.join(&self.target.inner);
        if path.is_file() && move_target.extension().is_none() {
            move_target.set_extension(&config.default_filetype);
        }
        debug!("Moving {:?} to {:?}", path, move_target);
        if path.is_file() || path.is_dir() {
            fs::create_dir_all(move_target.parent().unwrap())?;
            fs::rename(path, move_target)?;
        } else {
            return Err(CommandError::TargetMissing(self.target.clone()));
        }
        println!("Archive successful");
        Ok(())
    }
}
