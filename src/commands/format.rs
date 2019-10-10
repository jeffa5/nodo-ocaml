use log::*;
use std::fs;

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
        if nodo.metadata().target() == "" {
            return Err(CommandError("Nodo must exist to format".to_string()));
        }
        let path = file::build_path(&config, &nodo);
        let metadata = fs::metadata(&path)?;
        if metadata.is_dir() {
            return Err(CommandError(format!(
                "Can't format {} since it is a project",
                path.to_string_lossy()
            )));
        }
        F::write(
            &F::read(nodo, &mut fs::File::open(&path)?)?,
            &mut fs::File::create(&path)?,
        )?;

        Ok(())
    }
}
