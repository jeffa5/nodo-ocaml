use log::*;
use std::fs;
use std::process::Command as Cmd;

use crate::commands::{Command, CommandError};
use crate::config::Config;
use crate::files::NodoFile;
use crate::nodo::Nodo;
use crate::util::file;

pub struct Edit;

impl Command for Edit {
    /// Edit a current nodo in the editor
    fn exec<F: NodoFile>(config: Config, nodo: Nodo<F>) -> Result<(), CommandError> {
        trace!("Editing a nodo");
        // get the file location
        if nodo.metadata().target() == "" {
            return Err(CommandError("Nodo must exist to edit".to_string()));
        }
        let path = file::build_path(&config, &nodo);
        // launch the editor with that location
        let metadata = fs::metadata(&path)?;
        if metadata.is_dir() {
            return Err(CommandError(format!(
                "Can't edit {} since it is a project",
                path.to_string_lossy()
            )));
        }
        let editor = get_editor();
        let mut command = Cmd::new(editor);
        command.arg(path);
        debug!("Editor command is: {:?}", command);
        let status = command.status().expect("Failed to open editor");
        debug!("Editor finished with status: {}", status);
        Ok(())
    }
}

fn get_editor() -> String {
    match std::env::var("EDITOR") {
        Ok(editor) => editor,
        Err(_) => String::from("vi"),
    }
}
