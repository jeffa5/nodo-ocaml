use log::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub root_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub archive_dir: PathBuf,
    pub projects_delimeter: String,
    pub default_filetype: String,
    pub date_format: String,
    #[serde(default)]
    pub overview_ignore_dirs: Vec<String>,
    pub sort_tasks: bool,
}

impl std::default::Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().expect("Failed to get home dir");

        let root_dir = home.join(".nodo");
        let temp_dir = root_dir.join("temp");
        let archive_dir = root_dir.join("archive");

        let config = Config {
            root_dir,
            temp_dir,
            archive_dir,
            projects_delimeter: "/".to_string(),
            default_filetype: "md".to_string(),
            date_format: "%d-%m-%Y".to_string(),
            overview_ignore_dirs: Vec::new(),
            sort_tasks: true,
        };

        setup_dir(&config.root_dir);
        setup_dir(&config.temp_dir);
        setup_dir(&config.archive_dir);

        config
    }
}

impl Config {
    pub fn load() -> Self {
        let conf_path = dirs::config_dir().unwrap().join("nodo/nodo");
        debug!("Loading config from {:?}", conf_path);

        let mut conf =
            config::Config::try_from(&Self::default()).expect("Failed to convert the default conf");

        conf.merge(config::File::with_name(&conf_path.to_string_lossy()).required(false))
            .unwrap()
            .merge(config::Environment::with_prefix("NODO"))
            .unwrap();

        conf.try_into::<Config>()
            .expect("Failed to parse config file")
    }
}

fn setup_dir(p: &Path) {
    debug!("Config: creating dir {:?}", p);
    if !p.exists() {
        fs::create_dir_all(p).expect("Failed to create default dir")
    }
}
