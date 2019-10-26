use log::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub root_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub archive_dir: PathBuf,
    pub projects_delimeter: &'static str,
    pub default_filetype: &'static str,
    pub date_format: &'static str,
}

impl std::default::Default for Config {
    fn default() -> Self {
        let mut config = Config {
            root_dir: PathBuf::from(".nodo"),
            temp_dir: PathBuf::from("temp"),
            archive_dir: PathBuf::from("archive"),
            projects_delimeter: "/",
            default_filetype: "md",
            date_format: "%d-%m-%Y",
        };
        let home = dirs::home_dir().expect("Failed to get home dir");
        config.root_dir = home.join(config.root_dir);
        config.temp_dir = config.root_dir.join(config.temp_dir);
        config.archive_dir = config.root_dir.join(config.archive_dir);
        setup_dir(&config.root_dir);
        setup_dir(&config.temp_dir);
        setup_dir(&config.archive_dir);
        config
    }
}

fn setup_dir(p: &Path) {
    debug!("Config: creating dir {:?}", p);
    if !p.exists() {
        std::fs::create_dir_all(p).expect("Failed to create default dir")
    }
}
