use log::*;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::cli::Show;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::{Block, List, ListItem, NodoBuilder};
use crate::util::file::build_path;

impl Show {
    /// Show a project or nodo
    /// Accepts empty target, dir or file
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        debug!("target: {:?}", &self.target);
        let mut path = build_path(&config, &self.target, false);
        debug!("path: {:?}", &path);
        if self.target.is_empty() || self.target.last().unwrap() == "" {
            show_dir(&path)?;
            return Ok(());
        }
        if let Err(err) = path.metadata() {
            if std::io::ErrorKind::NotFound == err.kind() {
                if path.extension().is_none() {
                    path.set_extension(config.default_filetype);
                    debug!("path: {:?}", &path);
                }
            } else {
                return Err(err.into());
            }
        }
        match path.metadata() {
            Err(err) => {
                return Err(match err.kind() {
                    ErrorKind::NotFound => CommandError::TargetMissing(&self.target),
                    _ => err.into(),
                })
            }
            Ok(metadata) => {
                debug!("metadata: {:?}", &metadata);
                if metadata.is_dir() {
                    // show the contents of the directory
                    trace!("Target was a directory");
                    show_dir(&path)?;
                } else if metadata.is_file() {
                    // show the content of the nodo
                    trace!("Target was a file");
                    self.show_file(&config, &path)?;
                }
            }
        }
        Ok(())
    }

    fn show_file<'a>(
        &self,
        config: &Config,
        path: &std::path::Path,
    ) -> Result<(), CommandError<'a>> {
        let file_handler = files::get_file_handler(config.default_filetype);
        let nodo =
            file_handler.read(NodoBuilder::default(), &mut fs::File::open(path)?, &config)?;
        let mut builder = NodoBuilder::default();
        builder.tags(nodo.tags().to_vec());
        if let Some(date) = nodo.start_date() {
            builder.start_date(date);
        }
        if let Some(date) = nodo.due_date() {
            builder.due_date(date);
        }
        builder.title(nodo.title().clone());
        for block in nodo.blocks() {
            match block {
                Block::List(l) => {
                    builder.block(Block::List(filter_list(
                        &trim_list(&l, self.depth),
                        self.filter_complete,
                    )));
                }

                b => {
                    builder.block(b.clone());
                }
            }
        }
        if !cfg!(test) {
            file_handler.write(&builder.build(), &mut std::io::stdout(), &config)?;
        }
        Ok(())
    }
}

fn show_dir<'a>(path: &std::path::Path) -> Result<(), CommandError<'a>> {
    let contents = fs::read_dir(path)?;
    for entry in contents {
        let entry = entry.expect("Failed to get direntry");
        let filetype = entry.file_type()?;
        let mut prefix = "";
        if filetype.is_dir() {
            prefix = "P"
        } else if filetype.is_file() {
            prefix = "N"
        }
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
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target { target: Vec::new() },
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn empty_args_is_not_an_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                target: "".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn cant_show_non_existent_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            show.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testdir".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_show_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn cant_show_non_existent_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            show.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_show_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                target: "testfile".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn can_show_existing_file_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            filter_complete: None,
            depth: None,
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }
}