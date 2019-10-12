use log::*;
use std::fs;
use std::path::PathBuf;

use crate::cli::NodoOpts;
use crate::commands::{Command, CommandError};
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::NodoBuilder;
use crate::util::file::build_path;

pub struct List;

impl Command for List {
    fn exec(config: Config, nodo_opts: NodoOpts) -> Result<(), CommandError> {
        debug!("Listing with nodo options: {:#?}", nodo_opts);

        let path = build_path(&config, &nodo_opts.target);
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
            let file_handler = files::get_file_handler(config.default_filetype);
            let nodo = file_handler.read(NodoBuilder::default(), &mut fs::File::open(path)?)?;
            debug!("{:#?}", nodo);
            file_handler.write(&nodo, &mut std::io::stdout())?;
        }
        Ok(())
    }
}
