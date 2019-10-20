use log::*;
use std::fs;
use std::io::ErrorKind;
use walkdir::WalkDir;

use crate::cli::Format;
use crate::commands::CommandError;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::nodo::NodoBuilder;
use crate::util::file;

impl Format {
    /// Format a nodo
    /// Accepts an empty target or a dir or a file
    pub fn exec(&self, config: Config) -> Result<(), CommandError> {
        debug!("target: {:?}", &self.target);
        trace!("Formatting a nodo");
        // get the file location
        let mut path = file::build_path(&config, &self.target, false);
        debug!("path: {:?}", &path);
        let metadata = path.metadata();
        debug!("metadata: {:?}", &metadata);
        if let Err(err) = metadata {
            if std::io::ErrorKind::NotFound == err.kind() {
                if path.extension().is_none() {
                    path.set_extension(config.default_filetype);
                    debug!("path: {:?}", &path);
                }
            } else {
                return Err(err.into());
            }
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
                // if metadata was err then check if it was a not found error, if so then see if extension was there, if it was exit else add default extension and try again
                // if metadata was ok then see whether dir or file and format appropriately
                let handler = files::get_file_handler(config.default_filetype);
                if metadata.is_dir() {
                    for entry in WalkDir::new(&path) {
                        let entry = entry?;
                        if entry.file_type().is_file() {
                            debug!("Formatting {}", entry.path().to_string_lossy());
                            self.format(&handler, &entry.path(), &config)?
                        }
                    }
                } else if metadata.is_file() {
                    self.format(&handler, &path, &config)?
                }
            }
        }
        Ok(())
    }

    fn format<F: NodoFile>(
        &self,
        handler: &F,
        path: &std::path::Path,
        config: &Config,
    ) -> Result<(), CommandError> {
        if self.verbose {
            println!(
                "Formatting nodo: {}",
                path.strip_prefix(&config.root_dir).unwrap().display()
            )
        }
        if self.dry_run {
            handler.write(
                &handler.read(NodoBuilder::default(), &mut fs::File::open(&path)?, config)?,
                &mut std::io::stdout(),
                config,
            )?;
        } else {
            handler.write(
                &handler.read(NodoBuilder::default(), &mut fs::File::open(&path)?, config)?,
                &mut fs::File::create(&path)?,
                config,
            )?;
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
    fn no_args_is_ok() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target { target: Vec::new() },
        };
        assert_eq!(format.exec(config), Ok(()));
    }

    #[test]
    fn empty_args_is_ok() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                target: "".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(format.exec(config), Ok(()));
    }

    #[test]
    fn cant_format_nonexisting_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            format.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testdir".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_format_an_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                target: "testdir".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(format.exec(config), Ok(()));
    }

    #[test]
    fn cant_format_nonexisting_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(
            format.exec(config),
            Err(CommandError::TargetMissing(&Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            }))
        );
    }

    #[test]
    fn can_format_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                target: "testfile".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(format.exec(config), Ok(()));
    }

    #[test]
    fn can_format_existing_file_with_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::new();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                target: "testfile.md".split('/').map(String::from).collect(),
            },
        };
        assert_eq!(format.exec(config), Ok(()));
    }
}
