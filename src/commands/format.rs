use log::*;
use std::fs;
use walkdir::WalkDir;

use crate::commands::{Command, CommandError};
use crate::config::Config;
use crate::files::NodoFile;
use crate::nodo::Nodo;
use crate::util::file;

pub struct Format;

impl Command for Format {
    /// Format a nodo in place
    fn exec<F: NodoFile>(config: Config, nodo: Nodo<F>) -> Result<(), CommandError> {
        trace!("Formatting a nodo");
        // get the file location
        let path = file::build_path(&config, &nodo);
        let metadata = fs::metadata(&path)?;
        if metadata.is_dir() {
            for entry in WalkDir::new(&path) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    debug!("Formatting {}", entry.path().to_string_lossy());
                    format::<F>(&entry.path())?
                }
            }
        } else if metadata.is_file() {
            format::<F>(&path)?
        }
        Ok(())
    }
}

fn format<F: NodoFile>(path: &std::path::Path) -> Result<(), CommandError> {
    F::write(
        &F::read(Nodo::new(), &mut fs::File::open(&path)?)?,
        &mut fs::File::create(&path)?,
    )?;
    Ok(())
}
