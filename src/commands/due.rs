use colored::*;
use log::*;
use std::cmp::Reverse;
use std::fs;
use std::path::PathBuf;

use crate::cli::Due;
use crate::commands;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::Nodo;
use crate::nodo::TextItem;
use crate::util;

impl Due {
    /// Show the due nodos
    pub fn exec(&self, config: Config) -> commands::Result<()> {
        let mut due = Vec::new();
        get_nodos_due(&config, &config.root_dir, &mut due)?;

        due.sort_by_key(|n| Reverse(n.due_date()));

        for n in due {
            let title = n
                .title()
                .inner
                .iter()
                .map(|item| match item {
                    TextItem::PlainText(s) => s.to_string(),
                    TextItem::StyledText(s, _) => s.to_string(),
                    TextItem::Link(s, _) => s.to_string(),
                })
                .collect::<Vec<_>>()
                .join(" ");

            println!(
                "{} {}",
                n.due_date()
                    .unwrap()
                    .format(&config.date_format)
                    .to_string()
                    .bold(),
                title,
            )
        }

        Ok(())
    }
}

fn get_nodos_due(config: &Config, path: &PathBuf, due: &mut Vec<Nodo>) -> commands::Result<()> {
    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        let found_ignore_dirs = config.overview_ignore_dirs.iter().any(|d| {
            d == &entry
                .path()
                .strip_prefix(&config.root_dir)
                .unwrap()
                .to_string_lossy()
        });
        if util::is_hidden_dir(&config, &path, &entry.path()) || found_ignore_dirs {
            debug!("Ignoring: {:?}", entry);
            continue;
        }
        debug!("Found {:?} while walking", entry);

        if entry.file_type().unwrap().is_dir() {
            get_nodos_due(config, &entry.path(), due)?;
        } else if entry.file_type().unwrap().is_file() {
            let handler = files::get_file_handler(&config.default_filetype);
            let nodo = handler.read(&mut std::fs::File::open(entry.path())?, config)?;
            if nodo.due_date().is_some() {
                due.push(nodo);
            }
        }
    }
    Ok(())
}
