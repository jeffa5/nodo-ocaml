mod cli;
mod commands;
mod config;
mod files;
mod nodo;
mod util;

use cli::{Cli, SubCommand, Target};

use log::*;
use structopt::StructOpt;

fn main() {
    let opts = Cli::from_args();
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .quiet(opts.quiet)
        .verbosity(opts.log_level)
        .init()
        .expect("Failed to initialise logging");
    debug!("{:#?}", opts);
    // let mut config: config::Config = confy::load("nodo").expect("Failed to get config file");

    let mut config = config::Config::load();
    debug!("{:#?}", config);
    if let Some(ft) = opts.filetype {
        config.default_filetype = ft
    }
    let result = match opts.sub_command {
        None => {
            let overview = cli::Overview {
                target: Target::default(),
            };
            overview.exec(config)
        }
        Some(sub_command) => match sub_command {
            SubCommand::New(new) => new.exec(config),
            SubCommand::Show(show) => show.exec(config),
            SubCommand::Remove(remove) => remove.exec(config),
            SubCommand::Edit(edit) => edit.exec(config),
            SubCommand::Clean(clean) => clean.exec(config),
            SubCommand::Format(format) => format.exec(config),
            SubCommand::Overview(overview) => overview.exec(config),
            SubCommand::Archive(archive) => archive.exec(config),
        },
    };
    if let Err(err) = result {
        println!("{}", err)
    }
}
