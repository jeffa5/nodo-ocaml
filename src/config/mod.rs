pub struct Config {
    pub root_dir: std::path::PathBuf,
    pub projects_delimeter: &'static str,
    pub default_filetype: &'static str,
    pub date_format: &'static str,
}

impl Config {
    pub fn new() -> Config {
        Config {
            root_dir: get_root_dir(),
            projects_delimeter: "/",
            default_filetype: "md",
            date_format: "%d/%m/%Y",
        }
    }
}

fn get_root_dir() -> std::path::PathBuf {
    let mut root_dir = dirs::home_dir().expect("Failed to get home dir");
    root_dir.push("nodo");
    root_dir
}
