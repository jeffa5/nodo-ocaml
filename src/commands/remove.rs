use std::fs;

use crate::cli::Remove;
use crate::commands::CommandError;
use crate::config::Config;
use crate::util::file;

impl Remove {
    /// Remove a nodo if it exists
    pub fn exec(self, config: Config) -> Result<(), CommandError> {
        let path = file::build_path(&config, &self.target);
        let metadata = fs::metadata(&path)?;
        let res = if metadata.is_file() {
            fs::remove_file(&path)
        } else if metadata.is_dir() {
            fs::remove_dir(&path)
        } else {
            return Err("Not sure what type of file the target was".into());
        };
        match res {
            Ok(()) => {
                println!("Removed nodo: {}", self.target.join("/"));
            }
            Err(_) => println!("No such nodo to remove: {}", self.target.join("/")),
        }
        Ok(())
    }
}
