use log::*;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::cli::Overview;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::{Block, List, ListItem, NodoBuilder};
use crate::util::file::build_path;

impl Overview {
    pub fn exec(self, config: Config) -> Result<(), CommandError> {
        let path = build_path(&config, &self.target, true);
        if self.target.is_empty() {
            project_overview(&config, &path)?;
        } else {
            let metadata = fs::metadata(&path)?;
            if metadata.is_dir() {
                project_overview(&config, &path)?;
            } else if metadata.is_file() {
                unimplemented!()
            }
        }
        Ok(())
    }
}

fn project_overview(config: &Config, base_path: &Path) -> Result<(), CommandError> {
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
            let (complete, total) = get_num_complete(config, entry.path())?;
            let complete_string = if total > 0 {
                format!(" completed {}/{}", complete, total)
            } else {
                String::new()
            };
            println!(
                "{}Nodo: {}{} ({:.1}%)",
                "  ".repeat(depth + 1),
                entry
                    .path()
                    .strip_prefix(&base_path)
                    .unwrap()
                    .to_string_lossy(),
                complete_string,
                100. * f64::from(complete) / f64::from(total)
            );
        }
    }
    Ok(())
}

fn get_num_complete(config: &Config, path: &Path) -> Result<(u32, u32), CommandError> {
    let handler = files::get_file_handler(config.default_filetype);
    let nodo = handler.read(NodoBuilder::default(), &mut fs::File::open(&path)?, config)?;
    let mut total = 0;
    let mut complete = 0;
    for block in nodo.blocks() {
        match block {
            Block::List(List::Plain(items)) | Block::List(List::Numbered(items, _)) => {
                for item in items {
                    // not recursive so doesn't count sub-tasks
                    if let ListItem::Task(_, true, _) = item {
                        complete += 1;
                    }
                    total += 1;
                }
            }
            _ => (),
        }
    }
    Ok((complete, total))
}
