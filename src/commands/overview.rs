use log::*;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

use crate::cli::Overview;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::Nodo;
use crate::nodo::TextItem;
use crate::nodo::{Block, List, ListItem, NodoBuilder};
use crate::util::file::build_path;

impl Overview {
    /// Provide an overview of the target
    /// Accepts an empty target, dir or file
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        debug!("target: {:?}", &self.target);
        let mut path = build_path(&config, &self.target, false);
        debug!("path: {:?}", &path);
        let dirtrees = if self.target.is_empty() {
            dir_overview(&config, &path, 0)?
        } else {
            if let Err(err) = path.metadata() {
                if std::io::ErrorKind::NotFound == err.kind() {
                    if path.extension().is_none() {
                        path.set_extension(&config.default_filetype);
                        debug!("path: {:?}", &path);
                    }
                } else {
                    return Err(err.into());
                }
            }
            match path.metadata() {
                Err(err) => {
                    return Err(match err.kind() {
                        ErrorKind::NotFound => CommandError::TargetMissing(self.target.clone()),
                        _ => err.into(),
                    })
                }
                Ok(metadata) => {
                    if metadata.is_dir() {
                        dir_overview(&config, &path, 0)?
                    } else if metadata.is_file() {
                        let mut dirtree = DirTree::default();
                        file_overview(&config, &path, &mut dirtree)?;
                        debug!("{:?}", dirtree);
                        vec![dirtree]
                    } else {
                        Vec::new()
                    }
                }
            }
        };
        for dirtree in dirtrees {
            println!("{}", dirtree);
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
                "N: ({}) {}{}",
                self.path.file_name().unwrap().to_string_lossy(),
                self.title,
                complete_string
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

fn dir_overview(config: &Config, path: &Path, depth: usize) -> Result<Vec<DirTree>, CommandError> {
    let mut dirtrees = Vec::new();
    for entry in std::fs::read_dir(&path)? {
        // for entry in WalkDir::new(&path).min_depth(1).max_depth(1) {
        let entry = entry?;
        if (entry.path().starts_with(&config.temp_dir) && !path.starts_with(&config.temp_dir))
            || (entry.path().starts_with(&config.archive_dir)
                && !path.starts_with(&config.archive_dir))
        {
            debug!("Ignoring: {:?}", entry);
            continue;
        }
        debug!("Found {:?} while walking", entry);
        let mut dirtree = DirTree::default();
        dirtree.depth = depth;
        dirtree.path = entry.path().to_path_buf();

        if entry.file_type().unwrap().is_dir() {
            dirtree.children = dir_overview(config, &entry.path(), depth + 1)?;
            dirtree.total = dirtree.children.iter().map(|c| c.total).sum();
            dirtree.complete = dirtree.children.iter().map(|c| c.complete).sum();
        } else if entry.file_type().unwrap().is_file() {
            file_overview(config, &entry.path(), &mut dirtree)?
        }
        dirtrees.push(dirtree)
    }
    Ok(dirtrees)
}

fn file_overview(config: &Config, path: &Path, dirtree: &mut DirTree) -> Result<(), CommandError> {
    let handler = files::get_file_handler(&config.default_filetype);
    let nodo = handler.read(
        NodoBuilder::default(),
        &mut std::fs::File::open(path)?,
        config,
    )?;
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
    Ok(())
}

fn get_num_complete(nodo: &Nodo) -> Result<(u32, u32), CommandError> {
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
        let overview = Overview {
            target: Target::default(),
        };
        assert_eq!(overview.exec(config), Ok(()));
    }

    #[test]
    fn empty_args_is_not_an_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target::default(),
        };
        assert_eq!(overview.exec(config), Ok(()));
    }

    #[test]
    fn cant_overview_non_existent_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                inner: "testdir".to_string(),
            },
        };
        assert_eq!(
            overview.exec(config),
            Err(CommandError::TargetMissing(Target {
                inner: "testdir".to_string(),
            }))
        );
    }

    #[test]
    fn can_overview_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                inner: "testdir".to_string(),
            },
        };
        assert_eq!(overview.exec(config), Ok(()));
    }

    #[test]
    fn cant_overview_non_existent_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                inner: "testfile".to_string(),
            },
        };
        assert_eq!(
            overview.exec(config),
            Err(CommandError::TargetMissing(Target {
                inner: "testfile".to_string(),
            }))
        );
    }

    #[test]
    fn can_overview_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                inner: "testfile".to_string(),
            },
        };
        assert_eq!(overview.exec(config), Ok(()));
    }

    #[test]
    fn can_overview_existing_file_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                inner: "testfile.md".to_string(),
            },
        };
        assert_eq!(overview.exec(config), Ok(()));
    }
}
