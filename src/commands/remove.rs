use std::fs;

use crate::cli::Remove;
use crate::commands::CommandError;
use crate::config::Config;
use crate::util::file;

impl Remove {
    /// Remove a nodo if it exists
    pub fn exec(self, config: Config) -> Result<(), CommandError> {
        if self.target.is_empty() || self.target.last().unwrap() == "" {
            return Err(CommandError("Must provide a target".into()));
        }
        let path = file::build_path(&config, &self.target, true);
        let metadata = fs::metadata(&path)?;
        let res = if metadata.is_file() {
            fs::remove_file(&path)
        } else if metadata.is_dir() {
            fs::remove_dir(&path)
        } else {
            return Err("Not sure what type of file the target was".into());
        };
        match res {
            Ok(()) => {
                println!("Removed nodo: {}", self.target.join("/"));
            }
            Err(_) => println!("No such nodo to remove: {}", self.target.join("/")),
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
            target: Target { target: Vec::new() },
        };
        assert_eq!(
            remove.exec(config),
            Err(CommandError("Must provide a target".into()))
        );
    }

    #[test]
    fn empty_args_is_an_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            target: Target {
                target: "".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            remove.exec(config),
            Err(CommandError("Must provide a target".into()))
        );
    }

    #[test]
    fn cant_remove_non_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            remove.exec(config),
            Err(CommandError("Couldn't find target".into()))
        );
    }

    #[test]
    fn cant_remove_an_existing_dir() {
        // should need --force
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            remove.exec(config),
            Err(CommandError("Couldn't find target".into()))
        );
    }

    #[test]
    fn cant_remove_non_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            remove.exec(config),
            Err(CommandError("Couldn't find target".into()))
        );
    }

    #[test]
    fn can_remove_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let remove = Remove {
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
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(remove.exec(config), Ok(()));
    }
}
