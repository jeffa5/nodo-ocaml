use log::*;
use std::fs;

use crate::cli::New;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::NodoBuilder;
use crate::util::{file, project};

impl New {
    /// Create a new nodo with the given options
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        if self.target.is_empty() {
            return Err(CommandError::NoTarget);
        }
        if let Some(template) = &self.template {
            debug!("Template given: {}", template.to_string_lossy());
            let mut template_path = config.root_dir.join(template);
            if template_path.extension().is_none() {
                template_path.set_extension(&config.default_filetype);
            }
            if !template_path.is_file() {
                return Err(CommandError::Str(format!(
                    "Template '{}' isn't a nodo in the root directory",
                    template.to_string_lossy()
                )));
            }
        }
        // ensure the project exists
        project::make_dirs(&config, &self.target)?;
        // write the nodo to the default location
        let pb = file::build_path(&config, &self.target, true);
        let mut file = file::create_file(&pb)?;
        if let Some(template) = &self.template {
            let mut template_path = config.root_dir.join(template);
            if template_path.extension().is_none() {
                template_path.set_extension(&config.default_filetype);
            }
            debug!(
                "Copying from template {} to {}",
                template_path.to_string_lossy(),
                pb.to_string_lossy()
            );
            fs::copy(template_path, pb)?;
        } else {
            info!("Writing nodo to: {:?}", file);
            let handler = files::get_file_handler(&config.default_filetype);
            let nodo = NodoBuilder::default().build();
            handler.write(&nodo, &mut file, &config)?;
        }
        println!("Created a new nodo: {}", self.target);
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
        let new = New {
            template: None,
            target: Target::default(),
        };
        assert_eq!(new.exec(config), Err(CommandError::NoTarget));
    }

    #[test]
    fn empty_args_is_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let new = New {
            template: None,
            target: Target {
                inner: "".to_string(),
            },
        };
        assert_eq!(new.exec(config), Err(CommandError::NoTarget));
    }

    #[test]
    fn creates_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let new = New {
            template: None,
            target: Target {
                inner: "testdir/testfile".to_string(),
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
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let new = New {
            template: None,
            target: Target {
                inner: "testdir/testfile.md".to_string(),
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
