use log::*;
use std::env;
use std::io::ErrorKind;
use std::process::Command as Cmd;

use crate::cli::Edit;
use crate::commands::CommandError;
use crate::config::Config;
use crate::util::file;

impl Edit {
    /// Edit a current nodo in the editor
    /// Only accepts a file as the target
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        trace!("Editing a nodo");
        // get the file location
        if self.target.is_empty() || self.target.last().unwrap() == "" {
            return Err(CommandError::NoTarget);
        }
        let path = file::build_path(&config, &self.target, true);
        match path.metadata() {
            Err(err) => {
                return Err(match err.kind() {
                    ErrorKind::NotFound => CommandError::TargetMissing(&self.target),
                    _ => err.into(),
                })
            }
            Ok(metadata) => {
                if metadata.is_dir() {
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
            }
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
    use tempfile::tempdir;

    #[test]
    fn no_args_is_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let edit = Edit {
            target: Target { target: Vec::new() },
        };
        assert_eq!(edit.exec(config), Err(CommandError::NoTarget));
    }

    #[test]
    fn empty_args_is_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let edit = Edit {
            target: Target {
                target: "".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(edit.exec(config), Err(CommandError::NoTarget));
    }

    #[test]
    fn cant_edit_nonexistent_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let edit = Edit {
            target: Target {
                target: "testdir/testfile".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            edit.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testdir/testfile".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn cant_edit_nonexistent_file_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let edit = Edit {
            target: Target {
                target: "testdir/testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            edit.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testdir/testfile.md".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_edit_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let edit = Edit {
            target: Target {
                target: "testfile".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(edit.exec(config), Ok(()));
    }

    #[test]
    fn can_edit_existing_file_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let edit = Edit {
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(edit.exec(config), Ok(()));
    }
}
