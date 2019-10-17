use crate::cli::Remove;
use crate::commands::CommandError;
use crate::config::Config;
use crate::util::file;

impl Remove {
    /// Remove a nodo if it exists
    pub fn exec(self, config: Config) -> Result<(), CommandError> {
        match file::remove_file(&config, &self.nodo_opts.target) {
            Ok(()) => {
                println!("Removed nodo: {}", self.nodo_opts.target.join("/"));
            }
            Err(_) => println!(
                "No such nodo to remove: {}",
                self.nodo_opts.target.join("/")
            ),
        }
        Ok(())
    }
}
