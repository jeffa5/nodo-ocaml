use log::*;
use std::fs;

use crate::cli::Clean;
use crate::commands;
use crate::config::Config;

impl Clean {
    /// Cleans the temporary directory
    pub fn exec(&self, config: Config) -> commands::Result<()> {
        trace!("Cleaning");
        fs::remove_dir_all(&config.temp_dir)?;
        fs::create_dir_all(&config.temp_dir)?;
        Ok(())
    }
}
