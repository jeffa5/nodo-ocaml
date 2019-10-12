use crate::cli::NodoOpts;
use crate::commands::{Command, CommandError};
use crate::config::Config;
use crate::util::file;

pub struct Remove;

impl Command for Remove {
    /// Remove a nodo if it exists
    fn exec(config: Config, nodo_opts: NodoOpts) -> Result<(), CommandError> {
        match file::remove_file(&config, &nodo_opts.target) {
            Ok(()) => {
                println!("Removed nodo: {}", nodo_opts.target.join("/"));
            }
            Err(_) => println!("No such nodo to remove: {}", nodo_opts.target.join("/")),
        }
        Ok(())
    }
}
