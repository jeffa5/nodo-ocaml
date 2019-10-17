extern crate dirs;
extern crate log;
extern crate pulldown_cmark;
extern crate stderrlog;
extern crate structopt;

mod cli;
mod commands;
mod config;
mod files;
mod nodo;
mod util;

use cli::{Cli, SubCommand};
use config::Config;

use log::*;
use structopt::StructOpt;

fn main() {
    let opts = Cli::from_args();
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .quiet(opts.quiet)
        .verbosity(opts.verbose)
        .init()
        .expect("Failed to initialise logging");
    debug!("{:#?}", opts);
    let config = Config::new();
    let mut res = Ok(());
    match opts.sub_command {
        None => {
            let overview = cli::Overview {
                target: cli::Target::default(),
            };
            res = overview.exec(config)
        }
        Some(sub_command) => match sub_command {
            SubCommand::New(new) => res = new.exec(config),
            SubCommand::List(list) => res = list.exec(config),
            SubCommand::Remove(remove) => res = remove.exec(config),
            SubCommand::Edit(edit) => res = edit.exec(config),
            SubCommand::Format(format) => res = format.exec(config),
            SubCommand::Overview(overview) => res = overview.exec(config),
            SubCommand::Completions { shell } => {
                Cli::clap().gen_completions_to(
                    "nodo",
                    shell
                        .parse()
                        .expect("Failed to parse shell as a shell candidate"),
                    &mut std::io::stdout(),
                );
            }
        },
    }
    match res {
        Ok(()) => (),
        Err(err) => {
            warn!("Reached top level error: {:?}", err);
            println!("{}", err)
        }
    }
}
