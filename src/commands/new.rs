use log::*;

use crate::commands::{Command, CommandError};
use crate::config::Config;
use crate::files::NodoFile;
use crate::nodo::Nodo;
use crate::util::{file, project};

pub struct New;

impl Command for New {
    /// Create a new nodo with the given options
    fn exec<F: NodoFile>(config: Config, nodo: Nodo<F>) -> Result<(), CommandError> {
        // ensure the project exists
        project::make_dirs(&config, &nodo)?;
        // write the nodo to the default location
        let mut file = file::create_file(&config, &nodo)?;
        // get the default filetype (md for now)
        info!("Writing nodo to: {:?}", file);
        F::write(&nodo, &mut file)?;
        println!(
            "Created a new nodo: {}/{}",
            nodo.metadata().projects().join("/"),
            nodo.metadata().target()
        );
        Ok(())
    }
}
