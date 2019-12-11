use colored::*;
use log::*;
use std::fs;
use std::path::Path;

use crate::cli::Format;
use crate::commands;
use crate::config::Config;
use crate::files;
use crate::files::NodoFile;
use crate::util;

impl Format {
    /// Format a nodo
    /// Accepts an empty target or a dir or a file
    pub fn exec(&self, config: Config) -> commands::Result<()> {
        debug!("target: {:?}", &self.target);
        trace!("Formatting a nodo");
        let path = util::find_target(&config, &self.target)?;
        let handler = files::get_file_handler(&config.default_filetype);
        if path.is_dir() {
            let local = util::local_file(&config);
            if local.exists() {
                self.format(&config, &local, &handler)?
            }

            self.format_dir(&config, &path, &handler)?
        } else if path.is_file() {
            self.format(&config, &path, &handler)?
        }
        Ok(())
    }

    fn format_dir<F: NodoFile>(
        &self,
        config: &Config,
        path: &Path,
        handler: &F,
    ) -> commands::Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if util::is_hidden_dir(config, &path, &entry.path()) {
                debug!("Ignoring: {:?}", entry);
                continue;
            }
            let file_type = entry.file_type().unwrap();
            if file_type.is_file() {
                debug!("Formatting {}", entry.path().to_string_lossy());
                if let Err(err) = self.format(config, &entry.path(), handler) {
                    println!(
                        "Failed to format {}: {}",
                        entry.path().to_string_lossy(),
                        err
                    )
                }
            } else if file_type.is_dir() {
                self.format_dir(config, &entry.path(), handler)?
            }
        }
        Ok(())
    }

    fn format<F: NodoFile>(
        &self,
        config: &Config,
        path: &Path,
        handler: &F,
    ) -> commands::Result<()> {
        if self.verbose {
            println!(
                "Formatting nodo: {}",
                path.strip_prefix(&config.root_dir).unwrap().display()
            )
        }
        let mut nodo = handler.read(&mut fs::File::open(&path)?, config)?;
        if config.sort_tasks {
            trace!("Sorting tasks");
            nodo.sort_tasks()
        }
        if self.dry_run {
            println!(
                "{}{}",
                "Formatted nodo: ".bold(),
                path.strip_prefix(&config.root_dir)
                    .unwrap()
                    .to_string_lossy()
                    .bold()
            );
            println!();
            handler.write(&nodo, &mut std::io::stdout(), config)?;
            println!()
        } else {
            handler.write(&nodo, &mut fs::File::create(&path)?, config)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::Target;
    use crate::commands::CommandError;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn no_args_is_ok() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target::default(),
        };
        assert_eq!(format.exec(config), Ok(()));
    }

    #[test]
    fn empty_args_is_ok() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target::default(),
        };
        assert_eq!(format.exec(config), Ok(()));
    }

    #[test]
    fn cant_format_nonexisting_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                inner: "testdir".to_string(),
            },
        };
        assert_eq!(
            format.exec(config),
            Err(CommandError::TargetMissing(Target {
                inner: "testdir".to_string()
            }))
        );
    }

    #[test]
    fn can_format_an_existing_dir() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::create_dir(dir.path().join("testdir")).expect("Failed to create testdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                inner: "testdir".to_string(),
            },
        };
        assert_eq!(format.exec(config), Ok(()));
    }

    #[test]
    fn cant_format_nonexisting_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                inner: "testfile.md".to_string(),
            },
        };
        assert_eq!(
            format.exec(config),
            Err(CommandError::TargetMissing(Target {
                inner: "testfile.md".to_string()
            }))
        );
    }

    #[test]
    fn can_format_existing_file() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                inner: "testfile".to_string(),
            },
        };
        assert_eq!(format.exec(config), Ok(()));
    }

    #[test]
    fn can_format_existing_file_with_ext() {
        let dir = tempdir().expect("Couldn't make tempdir");
        std::fs::write(dir.path().join("testfile.md"), "").expect("Failed to create testfile");
        let mut config = Config::default();
        config.root_dir = std::path::PathBuf::from(dir.path());
        let format = Format {
            verbose: false,
            dry_run: false,
            target: Target {
                inner: "testfile.md".to_string(),
            },
        };
        assert_eq!(format.exec(config), Ok(()));
    }
}
