use log::*;
use std::fs;

use crate::cli::Remove;
use crate::commands::{self, CommandError};
use crate::config::Config;
use crate::util;

impl Remove {
    /// Remove a nodo if it exists
    /// Accepts a dir (with force) or a file
    pub fn exec(&self, config: Config) -> commands::Result<()> {
        if self.target.is_empty() {
            trace!("No target, trying to use local file");
            let local = util::local_file(&config);
            if local.exists() {
                debug!("Local: {:?}", local);
                trace!("Local exists");
                fs::remove_file(&local)?;
                return Ok(());
            } else {
                return Err(CommandError::NoTarget);
            }
        }
        let path = util::find_target(&config, &self.target)?;
        let res = if path.is_file() {
            trace!("found a file");
            fs::remove_file(&path)
        } else if path.is_dir() {
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
                if path.is_dir() {
                    println!("Removed project: {}", self.target);
                } else if path.is_file() {
                    println!("Removed nodo: {}", self.target);
                }
            }
            Err(_) => println!("No such nodo to remove: {}", self.target),
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
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target::default(),
        };
        assert_eq!(remove.exec(config), Err(CommandError::NoTarget));
    }

    #[test]
    fn empty_args_is_an_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target::default(),
        };
        assert_eq!(remove.exec(config), Err(CommandError::NoTarget));
    }

    #[test]
    fn cant_remove_non_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                inner: "testdir".to_string(),
            },
        };
        assert_eq!(
            remove.exec(config),
            Err(CommandError::TargetMissing(Target {
                inner: "testdir".to_string()
            }))
        );
    }

    #[test]
    fn cant_remove_an_existing_dir_without_force() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                inner: "testdir".to_string(),
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
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: true,
            target: Target {
                inner: "testdir".to_string(),
            },
        };
        assert_eq!(remove.exec(config), Ok(()));
    }

    #[test]
    fn cant_remove_non_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                inner: "testfile.md".to_string(),
            },
        };
        assert_eq!(
            remove.exec(config),
            Err(CommandError::TargetMissing(Target {
                inner: "testfile.md".to_string()
            }))
        );
    }

    #[test]
    fn can_remove_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                inner: "testfile".to_string(),
            },
        };
        assert_eq!(remove.exec(config), Ok(()));
    }

    #[test]
    fn can_remove_existing_file_with_extension() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            force: false,
            target: Target {
                inner: "testfile.md".to_string(),
            },
        };
        assert_eq!(remove.exec(config), Ok(()));
    }
}
