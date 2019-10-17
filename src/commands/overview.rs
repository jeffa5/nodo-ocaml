use log::*;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::cli::Overview;
use crate::commands::CommandError;
use crate::config::Config;
use crate::util::file::build_path;

impl Overview {
    pub fn exec(self, config: Config) -> Result<(), CommandError> {
        let path = build_path(&config, &self.nodo_opts.target);
        if self.nodo_opts.target.is_empty() {
            project_overview(&path)?;
        } else {
            let metadata = fs::metadata(&path)?;
            if metadata.is_dir() {
                project_overview(&path)?;
            } else if metadata.is_file() {
                unimplemented!()
            }
        }
        Ok(())
    }
}

fn project_overview(base_path: &Path) -> Result<(), CommandError> {
    let mut depth = 0;
    for entry in WalkDir::new(&base_path).min_depth(1) {
        let entry = entry?;
        debug!("Found {:?} while walking", entry);
        if entry.file_type().is_dir() {
            depth = entry
                .path()
                .strip_prefix(&base_path)
                .unwrap()
                .ancestors()
                .count()
                - 2;
            println!(
                "{}Project: {}",
                "  ".repeat(depth),
                entry
                    .path()
                    .strip_prefix(&base_path)
                    .unwrap()
                    .to_string_lossy()
            );
        } else if entry.file_type().is_file() {
            println!(
                "{}Nodo: {}",
                "  ".repeat(depth + 1),
                entry
                    .path()
                    .strip_prefix(&base_path)
                    .unwrap()
                    .to_string_lossy()
            );
        }
    }
    Ok(())
}
