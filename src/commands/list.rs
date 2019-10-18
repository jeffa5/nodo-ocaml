use log::*;
use std::fs;
use std::path::PathBuf;

use crate::cli::List;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::NodoBuilder;
use crate::util::file::build_path;

impl List {
    pub fn exec(self, config: Config) -> Result<(), CommandError> {
        dbg!(&self.target);
        let path = build_path(&config, &self.target, false);
        dbg!(&path);
        if self.target.is_empty() || self.target.last().unwrap() == "" {
            list_dir(&path)?;
        }
        let metadata = fs::metadata(&path)?;
        dbg!(&metadata);
        if metadata.is_dir() {
            // list the contents of the directory
            trace!("Target was a directory");
            list_dir(&path)?;
        } else if metadata.is_file() {
            // show the content of the nodo
            trace!("Target was a file");
            let file_handler = files::get_file_handler(config.default_filetype);
            let nodo =
                file_handler.read(NodoBuilder::default(), &mut fs::File::open(path)?, &config)?;
            debug!("{:#?}", nodo);
            if !cfg!(test) {
                file_handler.write(&nodo, &mut std::io::stdout(), &config)?;
            }
        }
        Ok(())
    }
}

fn list_dir(path: &std::path::Path) -> Result<(), CommandError> {
    let contents = fs::read_dir(path)?;
    for entry in contents {
        let entry = entry.expect("Failed to get direntry");
        let filetype = entry.file_type()?;
        let mut prefix = "";
        if filetype.is_dir() {
            prefix = "P"
        } else if filetype.is_file() {
            prefix = "N"
        }
        println!(
            "{} {}",
            prefix,
            PathBuf::from(entry.file_name()).to_string_lossy()
        )
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::Target;
    use tempfile::tempdir;

    #[test]
    fn no_args_is_not_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let list = List {
            target: Target { target: Vec::new() },
        };
        assert_eq!(list.exec(config), Ok(()));
    }

    #[test]
    fn empty_args_is_not_an_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let list = List {
            target: Target {
                target: "".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(list.exec(config), Ok(()));
    }

    #[test]
    fn cant_list_non_existent_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let list = List {
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert!(list.exec(config).is_err());
    }

    #[test]
    fn can_list_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let list = List {
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(list.exec(config), Ok(()));
    }

    #[test]
    fn cant_list_non_existent_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let list = List {
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            list.exec(config),
            Err(CommandError("Couldn't find target".into()))
        );
    }

    #[test]
    fn can_list_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let list = List {
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(list.exec(config), Ok(()));
    }
}
