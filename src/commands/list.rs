use log::*;
use std::fs;
use std::path::PathBuf;

use crate::commands::{Command, CommandError};
use crate::config::Config;
use crate::files::NodoFile;
use crate::nodo::Nodo;
use crate::util::file::build_path;

pub struct List;

impl Command for List {
    fn exec<F: NodoFile>(config: Config, mut nodo: Nodo<F>) -> Result<(), CommandError> {
        debug!("Listing with nodo: {:#?}", nodo);

        let path = build_path(&config, &nodo);
        // nodo files don't have extensions so can only have a dir or a file of this name, no need
        // to consider files with other names and extensions
        let metadata = fs::metadata(&path)?;

        if metadata.is_dir() {
            // list the contents of the directory
            trace!("Target was a directory");

            let contents = fs::read_dir(path)?;
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
        } else if metadata.is_file() {
            // show the content of the nodo
            trace!("Target was a file");
            nodo = F::read(nodo, &mut fs::File::open(path)?)?;
            debug!("{:#?}", nodo);
            F::write(&nodo, &mut std::io::stdout())?;
        }
        Ok(())
    }
}
