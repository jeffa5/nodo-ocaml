use log::*;
use std::fs;
use std::path;
use std::path::PathBuf;

use crate::cli::Show;
use crate::cli::Target;
use crate::commands;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::Nodo;
use crate::nodo::TextItem;
use crate::nodo::{Block, List, ListItem, NodoBuilder};
use crate::util;

impl Show {
    /// Show a project or nodo
    /// Accepts empty target, dir or file
    pub fn exec(&self, config: Config) -> commands::Result<()> {
        debug!("target: {:?}", &self.target);
        if self.target.is_empty() {
            let local = util::local_file(&config);
            let local_opt = if local.exists() { Some(local) } else { None };
            show_dir(&config, &config.root_dir, local_opt)?;
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
            show_dir(&config, &path, None)?;
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
                        self.complete,
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
                file_handler.write(&builder.build(config), &mut std::io::stdout(), config)?;
            }
        }
        Ok(())
    }
}

#[derive(Default, Debug)]
struct DirTree {
    depth: usize,
    complete: u32,
    total: u32,
    title: String,
    due_date: String,
    path: PathBuf,
    children: Vec<DirTree>,
}

impl std::fmt::Display for DirTree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.write_tree(f, "")
    }
}

impl DirTree {
    fn write_tree(&self, f: &mut std::fmt::Formatter, prefix: &str) -> Result<(), std::fmt::Error> {
        let complete_string = if self.total > 0 {
            format!(
                " [{}/{} ({:.1}%)]",
                self.complete,
                self.total,
                100. * f64::from(self.complete) / f64::from(self.total)
            )
        } else {
            String::new()
        };
        let due_date_string = if !self.due_date.is_empty() {
            format!(" [due: {}]", self.due_date)
        } else {
            String::new()
        };
        if self.path.is_dir() {
            write!(
                f,
                "P: {}{}",
                self.path.file_name().unwrap().to_string_lossy(),
                complete_string
            )?;
        } else if self.path.is_file() {
            write!(
                f,
                "N: ({}) {}{}{}",
                self.path.file_name().unwrap().to_string_lossy(),
                self.title,
                complete_string,
                due_date_string
            )?;
        }
        for (i, child) in self.children.iter().enumerate() {
            writeln!(f)?;
            let mut prefix = prefix.to_string();
            if i == self.children.len() - 1 {
                write!(f, "{}└─ ", prefix)?;
                prefix.push_str("   ");
            } else {
                write!(f, "{}├─ ", prefix)?;
                prefix.push_str("│  ");
            };
            child.write_tree(f, &prefix)?;
        }
        Ok(())
    }
}

fn show_dir(config: &Config, path: &path::Path, local: Option<PathBuf>) -> commands::Result<()> {
    if let Some(l) = local {
        let handler = files::get_file_handler(&config.default_filetype);
        let nodo = handler.read(&mut std::fs::File::open(&l)?, config)?;
        let (complete, total) = get_num_complete(&nodo)?;
        let complete_string = if total > 0 {
            format!(
                " [{}/{} ({:.1}%)]",
                complete,
                total,
                100. * f64::from(complete) / f64::from(total)
            )
        } else {
            String::new()
        };
        println!(
            "N: ({}) {}{}",
            l.file_name().unwrap().to_string_lossy(),
            nodo.title(),
            complete_string,
        );
    }
    let trees = show_dir_internal(config, path, 0)?;
    for tree in trees {
        println!("{}", tree);
    }
    Ok(())
}

fn show_dir_internal(
    config: &Config,
    path: &path::Path,
    depth: usize,
) -> commands::Result<Vec<DirTree>> {
    let mut dirtrees = Vec::new();
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
        let mut dirtree = DirTree::default();
        dirtree.depth = depth;
        dirtree.path = entry.path().to_path_buf();

        if entry.file_type().unwrap().is_dir() {
            dirtree.children = show_dir_internal(config, &entry.path(), depth + 1)?;
            dirtree.total = dirtree.children.iter().map(|c| c.total).sum();
            dirtree.complete = dirtree.children.iter().map(|c| c.complete).sum();
        } else if entry.file_type().unwrap().is_file() {
            if let Err(err) = file_overview(config, &entry.path(), &mut dirtree) {
                warn!(
                    "Failed to overview {}: {}",
                    entry.path().to_string_lossy(),
                    err
                )
            }
        }
        dirtrees.push(dirtree)
    }
    Ok(dirtrees)
}

fn file_overview(
    config: &Config,
    path: &path::Path,
    dirtree: &mut DirTree,
) -> commands::Result<()> {
    let handler = files::get_file_handler(&config.default_filetype);
    let nodo = handler.read(&mut std::fs::File::open(path)?, config)?;
    let (complete, total) = get_num_complete(&nodo)?;
    dirtree.complete = complete;
    dirtree.total = total;
    let title = nodo
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
    dirtree.title = title;
    if let Some(dd) = nodo.due_date() {
        dirtree.due_date = dd.format(&config.date_format).to_string()
    }
    Ok(())
}

fn get_num_complete(nodo: &Nodo) -> commands::Result<(u32, u32)> {
    let mut total = 0;
    let mut complete = 0;
    for block in nodo.blocks() {
        match block {
            Block::List(List::Plain(items)) | Block::List(List::Numbered(items, _)) => {
                for item in items {
                    debug!("Counting item: {:?}", item);
                    // not recursive so doesn't count sub-tasks
                    if let ListItem::Task(_, is_complete, _) = item {
                        if *is_complete {
                            complete += 1;
                        }
                        total += 1;
                    }
                }
            }
            _ => (),
        }
    }
    Ok((complete, total))
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

fn filter_list(list: &List, complete: Option<bool>) -> List {
    if let Some(filter_complete) = complete {
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
    use crate::commands::CommandError;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn no_args_is_not_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            complete: None,
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
            complete: None,
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
            complete: None,
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
            complete: None,
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
            complete: None,
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
            complete: None,
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
            complete: None,
            depth: None,
            target: Target {
                inner: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }
}
