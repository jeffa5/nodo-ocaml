use log::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub root_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub archive_dir: PathBuf,
    pub projects_delimeter: String,
    pub default_filetype: String,
    pub date_format: String,
    pub overview_ignore_dirs: Vec<String>,
}

impl std::default::Default for Config {
    fn default() -> Self {
        let mut config = Config {
            root_dir: PathBuf::from(".nodo"),
            temp_dir: PathBuf::from("temp"),
            archive_dir: PathBuf::from("archive"),
            projects_delimeter: "/".to_string(),
            default_filetype: "md".to_string(),
            date_format: "%d-%m-%Y".to_string(),
            overview_ignore_dirs: Vec::new(),
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

impl Config {
    pub fn load() -> Self {
        let conf_path = dirs::config_dir().unwrap().join("nodo/nodo.toml");
        debug!("Loading config from {:?}", conf_path);
        if !conf_path.is_file() {
            fs::create_dir_all(conf_path.parent().unwrap())
                .expect("Failed to make dirs for config");
            let mut conf_file = fs::File::create(&conf_path).expect("Failed to create config file");
            let conf_str = toml::to_string_pretty(&Self::default())
                .expect("Failed to serialise default config");
            conf_file
                .write_all(conf_str.as_bytes())
                .expect("Failed to write default config");
        }
        let mut conf_file = fs::File::open(&conf_path).expect("Failed to open config file");
        let mut s = String::new();
        conf_file
            .read_to_string(&mut s)
            .expect("Failed to read contents of config file");
        toml::from_str(&s).expect("Failed to load config as toml")
    }
}

fn setup_dir(p: &Path) {
    debug!("Config: creating dir {:?}", p);
    if !p.exists() {
        fs::create_dir_all(p).expect("Failed to create default dir")
    }
}
