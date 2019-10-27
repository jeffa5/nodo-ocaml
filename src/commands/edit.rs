use chrono::offset::Local;
use log::*;
use std::env;
use std::fs::File;
use std::process::Command as Cmd;

use crate::cli::Edit;
use crate::commands::{self, CommandError};
use crate::config::Config;

impl Edit {
    /// Edit a current nodo in the editor
    /// Only accepts a file as the target
    pub fn exec(&self, config: Config) -> commands::Result<()> {
        trace!("Editing a nodo");
        // get the file location
        let path = if self.temp {
            if !self.target.is_empty() {
                return Err(CommandError::Str(
                    "Can't edit a temporary nodo with a target".to_string(),
                ));
            }
            let s = Local::now().format("%F-%T").to_string();
            debug!("temp file name: {}", s);
            let filename = config.temp_dir.join(s);
            File::create(&filename)?;
            filename
        } else if self.target.is_empty() {
            return Err(CommandError::NoTarget);
        } else {
            self.target.build_path(&config, true)
        };
        if self.create && !path.exists() {
            File::create(&path)?;
        }
        debug!("Using path: {:?}", path);
        if !path.exists() {
            return Err(CommandError::TargetMissing(self.target.clone()));
        }
        if path.is_dir() {
            return Err(CommandError::Str(format!(
                "Can't edit {} since it is a project",
                path.to_string_lossy()
            )));
        }
        let mut command = get_editor();
        command.arg(path);
        debug!("Editor command is: {:?}", command);
        if !cfg!(test) {
            let status = command.status().expect("Failed to open editor");
            debug!("Editor finished with status: {}", status);
        }
        Ok(())
    }
}

fn get_editor() -> Cmd {
    let mut editor = "vi".to_string();
    if let Ok(e) = env::var("VISUAL") {
        editor = e
    } else if let Ok(e) = env::var("EDITOR") {
        editor = e
    }
    Cmd::new(editor)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::Target;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn no_args_is_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = PathBuf::from(dir.path());
        let edit = Edit {
            create: false,
            temp: false,
            target: Target::default(),
        };
        assert_eq!(edit.exec(config), Err(CommandError::NoTarget));
    }

    #[test]
    fn empty_args_is_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = PathBuf::from(dir.path());
        let edit = Edit {
            create: false,
            temp: false,
            target: Target::default(),
        };
        assert_eq!(edit.exec(config), Err(CommandError::NoTarget));
    }

    #[test]
    fn cant_edit_nonexistent_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = PathBuf::from(dir.path());
        let edit = Edit {
            create: false,
            temp: false,
            target: Target {
                inner: "testdir/testfile".to_string(),
            },
        };
        assert_eq!(
            edit.exec(config),
            Err(CommandError::TargetMissing(Target {
                inner: "testdir/testfile".to_string(),
            }))
        );
    }

    #[test]
    fn cant_edit_nonexistent_file_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = PathBuf::from(dir.path());
        let edit = Edit {
            create: false,
            temp: false,
            target: Target {
                inner: "testdir/testfile.md".to_string(),
            },
        };
        assert_eq!(
            edit.exec(config),
            Err(CommandError::TargetMissing(Target {
                inner: "testdir/testfile.md".to_string(),
            }))
        );
    }

    #[test]
    fn can_edit_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::default();
        config.root_dir = PathBuf::from(dir.path());
        let edit = Edit {
            create: false,
            temp: false,
            target: Target {
                inner: "testfile".to_string(),
            },
        };
        assert_eq!(edit.exec(config), Ok(()));
    }

    #[test]
    fn can_edit_existing_file_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::default();
        config.root_dir = PathBuf::from(dir.path());
        let edit = Edit {
            create: false,
            temp: false,
            target: Target {
                inner: "testfile.md".to_string(),
            },
        };
        assert_eq!(edit.exec(config), Ok(()));
    }
}
