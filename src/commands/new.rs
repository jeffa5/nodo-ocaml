use log::*;

use crate::cli::New;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::NodoBuilder;
use crate::util::{file, project};

impl New {
    /// Create a new nodo with the given options
    pub fn exec(self, config: Config) -> Result<(), CommandError> {
        // ensure the project exists
        project::make_dirs(&config, &self.target)?;
        // write the nodo to the default location
        let mut file = file::create_file(&config, &self.target)?;
        // get the default filetype (md for now)
        info!("Writing nodo to: {:?}", file);
        let handler = files::get_file_handler(config.default_filetype);
        let nodo = NodoBuilder::default().build();
        handler.write(&nodo, &mut file, &config)?;
        println!("Created a new nodo: {}", self.target.join("/"));
        Ok(())
    }
}
