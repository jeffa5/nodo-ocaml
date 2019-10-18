use log::*;
use std::env;
use std::io;
use std::process::Command as Cmd;

use crate::cli::Edit;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::util::file;

impl Edit {
    /// Edit a current nodo in the editor
    pub fn exec(self, config: Config) -> Result<(), CommandError> {
        trace!("Editing a nodo");
        // get the file location
        if self.target.is_empty() || self.target.last().unwrap() == "" {
            return Err(CommandError("Please provide a nodo to edit".to_string()));
        }
        let path = file::build_path(&config, &self.target, true);
        // launch the editor with that location
        let metadata = path.metadata();
        if let Err(err) = &metadata {
            if let io::ErrorKind::NotFound = err.kind() {
                return Err(CommandError("Nodo must exist in order to edit".into()));
            }
        }
        let metadata = metadata?;
        if metadata.is_dir() {
            return Err(CommandError(format!(
                "Can't edit {} since it is a project",
                path.to_string_lossy()
            )));
        }
        let handler = files::get_file_handler(config.default_filetype);
        let mut command = get_editor(handler.ext());
        command.arg(path);
        debug!("Editor command is: {:?}", command);
        let status = command.status().expect("Failed to open editor");
        debug!("Editor finished with status: {}", status);
        Ok(())
    }
}

fn get_editor(ext: &str) -> Cmd {
    let mut editor = "vi".to_string();
    if let Ok(e) = env::var("VISUAL") {
        editor = e
    } else if let Ok(e) = env::var("EDITOR") {
        editor = e
    }
    if ["vi", "vim", "nvim"].iter().any(|&x| editor == x) {
        let mut cmd = Cmd::new(editor);
        cmd.arg("-c").arg(format!("set ft={}", ext));
        return cmd;
    }
    Cmd::new(editor)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::Target;
    use tempfile::tempdir;

    #[test]
    fn no_args_is_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let edit = Edit {
            target: Target { target: Vec::new() },
        };
        assert!(edit.exec(config).is_err());
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
        assert!(edit.exec(config).is_err());
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
        assert!(edit.exec(config).is_err());
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
        assert!(edit.exec(config).is_err());
    }
}
