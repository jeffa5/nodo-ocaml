use log::*;

use crate::cli::New;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::NodoBuilder;
use crate::util::{file, project};

impl New {
    /// Create a new nodo with the given options
    pub fn exec(self, config: Config) -> Result<(), CommandError> {
        if self.target.is_empty() || self.target.last().unwrap() == "" {
            return Err(CommandError("Must have a target".into()));
        }
        // ensure the project exists
        project::make_dirs(&config, &self.target)?;
        // write the nodo to the default location
        let pb = file::build_path(&config, &self.target, true);
        let mut file = file::create_file(&pb)?;
        info!("Writing nodo to: {:?}", file);
        let handler = files::get_file_handler(config.default_filetype);
        let nodo = NodoBuilder::default().build();
        handler.write(&nodo, &mut file, &config)?;
        println!("Created a new nodo: {}", self.target.join("/"));
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
        let new = New {
            target: Target { target: Vec::new() },
        };
        assert_eq!(
            new.exec(config),
            Err(CommandError("Must have a target".into()))
        );
    }

    #[test]
    fn empty_args_is_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let new = New {
            target: Target {
                target: "".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            new.exec(config),
            Err(CommandError("Must have a target".into()))
        );
    }

    #[test]
    fn creates_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let new = New {
            target: Target {
                target: "testdir/testfile".split('/').map(String::from).collect(),
            },
        };
        new.exec(config).expect("Exec failed");
        // dir should now contain another dir and that should contain a file
        assert!(dir
            .path()
            .join("testdir")
            .metadata()
            .expect("Failed to get metadata for testdir")
            .is_dir());
        assert!(dir
            .path()
            .join("testdir/testfile.md")
            .metadata()
            .expect("Failed to get metadata for testfile")
            .is_file());
    }

    #[test]
    fn creates_file_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let new = New {
            target: Target {
                target: "testdir/testfile.md".split('/').map(String::from).collect(),
            },
        };
        new.exec(config).expect("Exec failed");
        // dir should now contain another dir and that should contain a file
        assert!(dir
            .path()
            .join("testdir")
            .metadata()
            .expect("Failed to get metadata for testdir")
            .is_dir());
        assert!(dir
            .path()
            .join("testdir/testfile.md")
            .metadata()
            .expect("Failed to get metadata for testfile")
            .is_file());
    }
}
