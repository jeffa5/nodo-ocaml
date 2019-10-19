use log::*;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::cli::Show;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::NodoBuilder;
use crate::util::file::build_path;

impl Show {
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        debug!("target: {:?}", &self.target);
        let path = build_path(&config, &self.target, false);
        debug!("path: {:?}", &path);
        if self.target.is_empty() || self.target.last().unwrap() == "" {
            show_dir(&path)?;
            return Ok(());
        }
        match path.metadata() {
            Err(err) => {
                return Err(match err.kind() {
                    ErrorKind::NotFound => CommandError::TargetMissing(&self.target),
                    _ => err.into(),
                })
            }
            Ok(metadata) => {
                debug!("metadata: {:?}", &metadata);
                if metadata.is_dir() {
                    // show the contents of the directory
                    trace!("Target was a directory");
                    show_dir(&path)?;
                } else if metadata.is_file() {
                    // show the content of the nodo
                    trace!("Target was a file");
                    show_file(&config, &path)?;
                }
            }
        }
        Ok(())
    }
}

fn show_dir<'a>(path: &std::path::Path) -> Result<(), CommandError<'a>> {
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

fn show_file<'a>(config: &Config, path: &std::path::Path) -> Result<(), CommandError<'a>> {
    let file_handler = files::get_file_handler(config.default_filetype);
    let nodo = file_handler.read(NodoBuilder::default(), &mut fs::File::open(path)?, &config)?;
    debug!("{:#?}", nodo);
    if !cfg!(test) {
        file_handler.write(&nodo, &mut std::io::stdout(), &config)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::Target;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn no_args_is_not_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            target: Target { target: Vec::new() },
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn empty_args_is_not_an_error() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            target: Target {
                target: "".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn cant_show_non_existent_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            show.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testdir".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_show_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }

    #[test]
    fn cant_show_non_existent_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            show.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_show_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let show = Show {
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(show.exec(config), Ok(()));
    }
}
