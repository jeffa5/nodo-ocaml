use log::*;
use std::fs;
use std::io::ErrorKind;
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
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        debug!("target: {:?}", &self.target);
        let path = build_path(&config, &self.target, false);
        debug!("path: {:?}", &path);
        if self.target.is_empty() {
            project_overview(&config, &path)?;
            return Ok(());
        } else {
            match path.metadata() {
                Err(err) => {
                    return Err(match err.kind() {
                        ErrorKind::NotFound => CommandError::TargetMissing(&self.target),
                        _ => err.into(),
                    })
                }
                Ok(metadata) => {
                    if metadata.is_dir() {
                        project_overview(&config, &path)?;
                    } else if metadata.is_file() {
                        unimplemented!()
                    }
                }
            }
        }
        Ok(())
    }
}

fn project_overview<'a>(config: &Config, base_path: &Path) -> Result<(), CommandError<'a>> {
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
                format!(
                    " completed {}/{} ({:.1}%)",
                    complete,
                    total,
                    100. * f64::from(complete) / f64::from(total)
                )
            } else {
                String::new()
            };
            println!(
                "{}Nodo: {}{}",
                "  ".repeat(depth + 1),
                entry
                    .path()
                    .strip_prefix(&base_path)
                    .unwrap()
                    .to_string_lossy(),
                complete_string,
            );
        }
    }
    Ok(())
}

fn get_num_complete<'a>(config: &Config, path: &Path) -> Result<(u32, u32), CommandError<'a>> {
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
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            overview.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            }))
        );
    }

    // #[test]
    // fn can_overview_existing_file() {
    //     let dir = tempdir().expect("Couldn't make tempdir");
    //     std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
    //     let mut config = Config::new();
    //     config.root_dir = std::path::PathBuf::from(dir.path());
    //     let overview = Overview {
    //         target: Target {
    //             target: "testfile.md".split('/').map(String::from).collect(),
    //         },
    //     };
    //     assert_eq!(overview.exec(config), Ok(()));
    // }
}
