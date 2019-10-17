use log::*;
use std::env;
use std::fs;
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
        if self.target.target.is_empty() {
            return Err(CommandError("Nodo must exist to edit".to_string()));
        }
        let path = file::build_path(&config, &self.target.target);
        // launch the editor with that location
        let metadata = fs::metadata(&path)?;
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
