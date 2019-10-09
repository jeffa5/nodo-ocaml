use log::*;
use std::path::PathBuf;

use crate::commands::{Command, CommandError};
use crate::config::Config;
use crate::files::NodoFile;
use crate::nodo::Nodo;
use crate::util::file::{build_filepath, list_dir};

pub struct List;

impl Command for List {
    fn exec<F: NodoFile>(config: Config, nodo: Nodo<F>) -> Result<(), CommandError> {
        debug!("Listing with nodo: {:?}", nodo);
        // get the files and projects listed in the folder given
        if nodo.metadata().filename() != "" {
            trace!("Filename wasn't empty");
            // read the nodo file if it exists and print it for now
            let pb = build_filepath(&config, &nodo);
            let nodo = F::read(nodo, &mut std::fs::File::open(pb)?)?;
            debug!("{:#?}", nodo);
            F::write(&nodo, &mut std::io::stdout())?;
            Ok(())
        } else {
            trace!("Filename was empty, listing directory");
            // just show the directory
            let contents = list_dir(&config, &nodo);
            for entry in contents {
                let entry = entry.expect("Failed to get direntry");
                let filetype = entry.file_type()?;
                let mut prefix = "";
                if filetype.is_dir() {
                    prefix = "P "
                } else if filetype.is_file() {
                    prefix = "N "
                }
                println!(
                    "{} {}",
                    prefix,
                    PathBuf::from(entry.file_name())
                        .file_stem()
                        .expect("Failed to have a name for listed direntry")
                        .to_string_lossy()
                )
            }
            Ok(())
        }
    }
}
