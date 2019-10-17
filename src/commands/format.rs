use log::*;
use std::fs;
use walkdir::WalkDir;

use crate::cli::Format;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::NodoBuilder;
use crate::util::file;

impl Format {
    /// Format a nodo
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        trace!("Formatting a nodo");
        // get the file location
        let path = file::build_path(&config, &self.nodo_opts.target);
        let metadata = fs::metadata(&path)?;
        let handler = files::get_file_handler(config.default_filetype);
        if metadata.is_dir() {
            for entry in WalkDir::new(&path) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    debug!("Formatting {}", entry.path().to_string_lossy());
                    self.format(&handler, &entry.path())?
                }
            }
        } else if metadata.is_file() {
            self.format(&handler, &path)?
        }
        Ok(())
    }

    fn format<F: NodoFile>(&self, handler: &F, path: &std::path::Path) -> Result<(), CommandError> {
        if self.dry_run {
            handler.write(
                &handler.read(NodoBuilder::default(), &mut fs::File::open(&path)?)?,
                &mut std::io::stdout(),
            )?;
        } else {
            handler.write(
                &handler.read(NodoBuilder::default(), &mut fs::File::open(&path)?)?,
                &mut fs::File::create(&path)?,
            )?;
        }
        Ok(())
    }
}
