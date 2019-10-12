use log::*;
use std::fs;
use walkdir::WalkDir;

use crate::cli::NodoOpts;
use crate::commands::{Command, CommandError};
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::NodoBuilder;
use crate::util::file;

pub struct Format;

impl Command for Format {
    /// Format a nodo in place
    fn exec(config: Config, nodo_opts: NodoOpts) -> Result<(), CommandError> {
        trace!("Formatting a nodo");
        // get the file location
        let path = file::build_path(&config, &nodo_opts.target);
        let metadata = fs::metadata(&path)?;
        let handler = files::get_file_handler(config.default_filetype);
        if metadata.is_dir() {
            for entry in WalkDir::new(&path) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    debug!("Formatting {}", entry.path().to_string_lossy());
                    format(&handler, &entry.path())?
                }
            }
        } else if metadata.is_file() {
            format(&handler, &path)?
        }
        Ok(())
    }
}

fn format<F: NodoFile>(handler: &F, path: &std::path::Path) -> Result<(), CommandError> {
    handler.write(
        &handler.read(NodoBuilder::default(), &mut fs::File::open(&path)?)?,
        &mut fs::File::create(&path)?,
    )?;
    Ok(())
}
