use std::path::{Path, PathBuf};

pub struct Config {
    pub root_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub projects_delimeter: &'static str,
    pub default_filetype: &'static str,
    pub date_format: &'static str,
}

impl Config {
    pub fn new() -> Config {
        let config = Config {
            root_dir: PathBuf::from(".nodo"),
            temp_dir: PathBuf::from("temp"),
            projects_delimeter: "/",
            default_filetype: "md",
            date_format: "%d/%m/%Y",
        };
        let home = dirs::home_dir().expect("Failed to get home dir");
        setup_dir(&home.join(&config.root_dir));
        setup_dir(&home.join(&config.temp_dir));
        config
    }
}

fn setup_dir(p: &Path) {
    if !p.exists() {
        std::fs::create_dir_all(p).expect("Failed to create default dir")
    }
}
