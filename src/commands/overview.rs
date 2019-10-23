use log::*;
use std::io::ErrorKind;
use std::path::Path;
use walkdir::WalkDir;

use crate::cli::Overview;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::Nodo;
use crate::nodo::{Block, List, ListItem, NodoBuilder};
use crate::util::file::build_path;

impl Overview {
    /// Provide an overview of the target
    /// Accepts an empty target, dir or file
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        debug!("target: {:?}", &self.target);
        let mut path = build_path(&config, &self.target, false);
        debug!("path: {:?}", &path);
        if self.target.is_empty() {
            dir_overview(&config, &path)?;
            return Ok(());
        } else {
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
                    if metadata.is_dir() {
                        dir_overview(&config, &path)?;
                    } else if metadata.is_file() {
                        let overview = file_overview(&config, &path)?;
                        println!("{}", overview);
                    }
                }
            }
        }
        Ok(())
    }
}

fn dir_overview<'a>(config: &Config, base_path: &Path) -> Result<(), CommandError<'a>> {
    for entry in WalkDir::new(&base_path).min_depth(1) {
        let entry = entry?;
        if entry.path().starts_with(&config.temp_dir) {
            debug!("Ignoring: {:?}", entry);
            continue;
        }
        debug!("Found {:?} while walking", entry);
        let depth = entry
            .path()
            .strip_prefix(&base_path)
            .unwrap()
            .ancestors()
            .count()
            - 2;
        let indent = " ".repeat(depth);
        if entry.file_type().is_dir() {
            println!(
                "{}P: {}",
                indent,
                entry.path().file_name().unwrap().to_string_lossy()
            );
        } else if entry.file_type().is_file() {
            let overview = file_overview(config, entry.path())?;
            println!(
                "{}N: {}{}",
                indent,
                entry.path().file_name().unwrap().to_string_lossy(),
                overview
            );
        }
    }
    Ok(())
}

fn file_overview<'a>(config: &Config, path: &Path) -> Result<String, CommandError<'a>> {
    let handler = files::get_file_handler(config.default_filetype);
    let nodo = handler.read(
        NodoBuilder::default(),
        &mut std::fs::File::open(path)?,
        config,
    )?;
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
    Ok(complete_string)
}

fn get_num_complete<'a>(nodo: &Nodo) -> Result<(u32, u32), CommandError<'a>> {
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
        let overview = Overview {
            target: Target { target: Vec::new() },
        };
        assert_eq!(overview.exec(config), Ok(()));
    }

    #[test]
    fn empty_args_is_not_an_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                target: "".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(overview.exec(config), Ok(()));
    }

    #[test]
    fn cant_overview_non_existent_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            overview.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testdir".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_overview_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(overview.exec(config), Ok(()));
    }

    #[test]
    fn cant_overview_non_existent_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                target: "testfile".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            overview.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testfile".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_overview_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                target: "testfile".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(overview.exec(config), Ok(()));
    }

    #[test]
    fn can_overview_existing_file_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let overview = Overview {
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(overview.exec(config), Ok(()));
    }
}
