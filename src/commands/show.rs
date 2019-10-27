use log::*;
use std::fs;
use std::path::PathBuf;

use crate::cli::Show;
use crate::cli::Target;
use crate::commands::{self, CommandError};
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::{Block, List, ListItem, NodoBuilder};
use crate::util;

impl Show {
    /// Show a project or nodo
    /// Accepts empty target, dir or file
    pub fn exec(&self, config: Config) -> commands::Result<()> {
        debug!("target: {:?}", &self.target);
        if self.target.is_empty() {
            show_dir(&config, &config.root_dir)?;
            return Ok(());
        }
        let parts: Vec<String> = self.target.splitn(2, '#').map(String::from).collect();
        debug!("Parts: {:?}", parts);
        let path = util::find_target(
            &config,
            &Target {
                inner: parts.first().unwrap().to_string(),
            },
        )?;
        if path.is_dir() {
            // show the contents of the directory
            trace!("Target was a directory");
            show_dir(&config, &path)?;
        } else if path.is_file() {
            // show the content of the nodo
            trace!("Target was a file");
            self.show_file(&config, &path, parts.get(1).unwrap_or(&String::new()))?;
        }
        Ok(())
    }

    fn show_file(
        &self,
        config: &Config,
        path: &std::path::Path,
        header: &str,
    ) -> commands::Result<()> {
        let file_handler = files::get_file_handler(&config.default_filetype);
        let nodo = file_handler.read(&mut fs::File::open(path)?, &config)?;
        let mut builder = NodoBuilder::default();
        builder.tags(nodo.tags().to_vec());
        if let Some(date) = nodo.start_date() {
            builder.start_date(date);
        }
        if let Some(date) = nodo.due_date() {
            builder.due_date(date);
        }
        let mut header_matched = if header.is_empty() { None } else { Some(false) };
        let mut header_count = 1;
        let header_index = header.parse::<u32>();
        if nodo
            .title()
            .to_string()
            .to_lowercase()
            .starts_with(&header.to_lowercase())
        {
            header_matched = Some(true)
        }
        if let Ok(index) = header_index {
            if index == 1 {
                header_matched = Some(true)
            }
        }
        if header_matched.is_none() || header_matched.unwrap() {
            builder.title(nodo.title().clone());
        }
        for block in nodo.blocks() {
            match block {
                Block::Heading(t, l) => {
                    if let Some(matched) = header_matched {
                        if matched {
                            break;
                        }
                        header_count += 1;
                        if t.to_string()
                            .to_lowercase()
                            .starts_with(&header.to_lowercase())
                        {
                            header_matched = Some(true)
                        }
                        if let Ok(index) = header_index {
                            if index == header_count {
                                header_matched = Some(true)
                            }
                        }
                    }
                    builder.block(Block::Heading(t.clone(), *l));
                }
                Block::List(l) => {
                    if let Some(matched) = header_matched {
                        if !matched {
                            continue;
                        }
                    }
                    builder.block(Block::List(filter_list(
                        &trim_list(&l, self.depth),
                        self.filter_complete,
                    )));
                }
                b => {
                    if let Some(matched) = header_matched {
                        if !matched {
                            continue;
                        }
                    }
                    builder.block(b.clone());
                }
            }
        }
        if !cfg!(test) {
            if header_matched.is_some() && !header_matched.unwrap() {
                println!("Failed to match a header")
            } else {
                file_handler.write(&builder.build(), &mut std::io::stdout(), &config)?;
            }
        }
        Ok(())
    }
}

fn show_dir(config: &Config, path: &std::path::Path) -> commands::Result<()> {
    let contents = fs::read_dir(path)?;
    for entry in contents {
        let entry = entry.expect("Failed to get direntry");
        if util::is_hidden_dir(&config, &path, &entry.path()) {
            debug!("Ignoring: {:?}", entry);
            continue;
        }
        let filetype = entry.file_type()?;
        let prefix = if filetype.is_dir() {
            "P"
        } else if filetype.is_file() {
            "N"
        } else {
            return Err(CommandError::Str(format!(
                "Failed to determine filetype of {:?}",
                entry
            )));
        };
        println!(
            "{} {}",
            prefix,
            PathBuf::from(entry.file_name()).to_string_lossy()
        )
    }
    Ok(())
}

fn trim_list(list: &List, depth: Option<u32>) -> List {
    if let Some(depth) = depth {
        debug!("Trimming list with depth: {}", depth);
        let mut root_items = Vec::new();
        match list {
            List::Plain(items) | List::Numbered(items, _) => {
                for item in items {
                    match item {
                        ListItem::Text(blocks, child) => root_items.push(if depth > 0 {
                            match child {
                                Some(sublist) => ListItem::Text(
                                    blocks.to_vec(),
                                    Some(trim_list(&sublist, Some(depth - 1))),
                                ),
                                None => ListItem::Text(blocks.to_vec(), None),
                            }
                        } else {
                            ListItem::Text(blocks.to_vec(), None)
                        }),
                        ListItem::Task(blocks, complete, child) => root_items.push(if depth > 0 {
                            match child {
                                Some(sublist) => ListItem::Task(
                                    blocks.to_vec(),
                                    *complete,
                                    Some(trim_list(&sublist, Some(depth - 1))),
                                ),
                                None => ListItem::Task(blocks.to_vec(), *complete, None),
                            }
                        } else {
                            ListItem::Task(blocks.to_vec(), *complete, None)
                        }),
                    }
                }
            }
        }
        match list {
            List::Plain(_) => List::Plain(root_items),
            List::Numbered(_, index) => List::Numbered(root_items, *index),
        }
    } else {
        list.clone()
    }
}

fn filter_list(list: &List, filter_complete: Option<bool>) -> List {
    if let Some(filter_complete) = filter_complete {
        debug!("Filtering list with complete: {}", filter_complete);
        let mut root_items = Vec::new();
        match list {
            List::Plain(items) | List::Numbered(items, _) => {
                for item in items {
                    match item {
                        ListItem::Text(blocks, child) => root_items.push(ListItem::Text(
                            blocks.to_vec(),
                            child
                                .as_ref()
                                .map(|sublist| filter_list(&sublist, Some(filter_complete))),
                        )),
                        ListItem::Task(blocks, complete, child) => {
                            if filter_complete == *complete {
                                root_items.push(ListItem::Task(
                                    blocks.to_vec(),
                                    *complete,
                                    child.as_ref().map(|sublist| {
                                        filter_list(&sublist, Some(filter_complete))
                                    }),
                                ))
                            }
                        }
                    }
                }
            }
        }
        match list {
            List::Plain(_) => List::Plain(root_items),
            List::Numbered(_, index) => List::Numbered(root_items, *index),
        }
    } else {
        list.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::Target;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn no_args_is_not_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target::default(),
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn empty_args_is_not_an_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                inner: "".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn cant_show_non_existent_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                inner: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            show.exec(config),
            Err(CommandError::TargetMissing(Target {
                inner: "testdir".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_show_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                inner: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn cant_show_non_existent_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                inner: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            show.exec(config),
            Err(CommandError::TargetMissing(Target {
                inner: "testfile.md".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_show_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                inner: "testfile".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn can_show_existing_file_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                inner: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }
}
