use log::*;
use std::fs;
use std::io::ErrorKind;

use crate::cli::Remove;
use crate::commands::CommandError;
use crate::config::Config;
use crate::util::file;

impl Remove {
    /// Remove a nodo if it exists
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        if self.target.is_empty() || self.target.last().unwrap() == "" {
            return Err(CommandError::NoTarget);
        }
        let mut path = file::build_path(&config, &self.target, false);
        debug!("path: {:?}", &path);
        let metadata = path.metadata();
        debug!("metadata: {:?}", &metadata);
        if let Err(err) = metadata {
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
                let res = if metadata.is_file() {
                    trace!("found a file");
                    fs::remove_file(&path)
                } else if metadata.is_dir() {
                    trace!("found a dir");
                    if self.force {
                        fs::remove_dir_all(&path)
                    } else {
                        return Err(CommandError::Str(format!(
                            "'{}' is a directory, can't remove without '-f'",
                            self.target
                        )));
                    }
                } else {
                    return Err("Not sure what type of file the target was".into());
                };
                match res {
                    Ok(()) => {
                        if metadata.is_dir() {
                            println!("Removed project: {}", self.target);
                        } else if metadata.is_file() {
                            println!("Removed nodo: {}", self.target);
                        }
                    }
                    Err(_) => println!("No such nodo to remove: {}", self.target),
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::Target;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn no_args_is_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target { target: Vec::new() },
        };
        assert_eq!(remove.exec(config), Err(CommandError::NoTarget));
    }

    #[test]
    fn empty_args_is_an_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                target: "".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(remove.exec(config), Err(CommandError::NoTarget));
    }

    #[test]
    fn cant_remove_non_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            remove.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: vec!["testdir".to_string()]
            }))
        );
    }

    #[test]
    fn cant_remove_an_existing_dir_without_force() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            remove.exec(config),
            Err(CommandError::Str(
                "'testdir' is a directory, can't remove without '-f'".into()
            ))
        );
    }

    #[test]
    fn can_remove_an_existing_dir_with_force() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: true,
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(remove.exec(config), Ok(()));
    }

    #[test]
    fn cant_remove_non_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            remove.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: vec!["testfile.md".to_string()]
            }))
        );
    }

    #[test]
    fn can_remove_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                target: "testfile".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(remove.exec(config), Ok(()));
    }

    #[test]
    fn can_remove_existing_file_with_extension() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(remove.exec(config), Ok(()));
    }
}
