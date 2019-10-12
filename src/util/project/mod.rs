use crate::config::Config;
use log::*;
use std::path::PathBuf;

pub fn make_dirs(config: &Config, projects: &[String]) -> std::io::Result<()> {
    let mut pb = PathBuf::from(&config.root_dir);
    projects.iter().for_each(|project| pb.push(project));
    info!("Creating the project dirs: {:?}", pb);
    std::fs::create_dir_all(pb)
}
