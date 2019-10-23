use crate::config::Config;
use log::*;

pub fn make_dirs(config: &Config, target: &str) -> std::io::Result<()> {
    let pb = config.root_dir.join(target);
    let projects = pb.parent().unwrap();
    info!("Creating the project dirs: {:?}", projects);
    std::fs::create_dir_all(projects)
}
