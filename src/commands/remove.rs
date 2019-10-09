use crate::commands::{Command, CommandError};
use crate::config::Config;
use crate::files::NodoFile;
use crate::nodo::Nodo;
use crate::util::file;

pub struct Remove;

impl Command for Remove {
    /// Remove a nodo if it exists
    fn exec<F: NodoFile>(config: Config, nodo: Nodo<F>) -> Result<(), CommandError> {
        match file::remove_file(&config, &nodo) {
            Ok(()) => {
                println!(
                    "Removed nodo: {}/{}",
                    nodo.metadata().projects().join("/"),
                    nodo.metadata().filename()
                );
            }
            Err(_) => println!(
                "No such nodo to remove: {}/{}",
                nodo.metadata().projects().join("/"),
                nodo.metadata().filename()
            ),
        }
        Ok(())
    }
}
