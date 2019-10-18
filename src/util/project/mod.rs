use crate::config::Config;
use log::*;
use std::path::PathBuf;

pub fn make_dirs(config: &Config, target: &[String]) -> std::io::Result<()> {
    let mut pb = PathBuf::from(&config.root_dir);
    target.iter().enumerate().for_each(|(i, project)| {
        if i != target.len() - 1 {
            pb.push(project)
        }
    });
    info!("Creating the project dirs: {:?}", pb);
    std::fs::create_dir_all(pb)
}
