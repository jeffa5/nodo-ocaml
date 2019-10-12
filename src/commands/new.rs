use log::*;

use crate::cli::NodoOpts;
use crate::commands::{Command, CommandError};
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::NodoBuilder;
use crate::util::{file, project};

pub struct New;

impl Command for New {
    /// Create a new nodo with the given options
    fn exec(config: Config, nodo_opts: NodoOpts) -> Result<(), CommandError> {
        // ensure the project exists
        project::make_dirs(&config, &nodo_opts.target)?;
        // write the nodo to the default location
        let mut file = file::create_file(&config, &nodo_opts.target)?;
        // get the default filetype (md for now)
        info!("Writing nodo to: {:?}", file);
        let handler = files::get_file_handler(config.default_filetype);
        let nodo = NodoBuilder::from(&nodo_opts).build();
        handler.write(&nodo, &mut file)?;
        println!("Created a new nodo: {}", nodo_opts.target.join("/"));
        Ok(())
    }
}
