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
        .verbosity(opts.log_level)
        .init()
        .expect("Failed to initialise logging");
    debug!("{:#?}", opts);
    let config = Config::new();
    if opts.sub_command.is_some() && !opts.target.is_empty() {
        println!("Can't specify a target here and a subcommand");
        return;
    }
    let result = match opts.sub_command {
        None => {
            let path = util::file::build_path(&config, &opts.target, true);
            if !path.exists() {
                Cli::clap().print_help().unwrap();
                return;
            }
            let overview = cli::Overview {
                target: opts.target,
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
            SubCommand::Completions { shell } => {
                Cli::clap().gen_completions_to(
                    "nodo",
                    shell
                        .parse()
                        .expect("Failed to parse shell as a shell candidate"),
                    &mut std::io::stdout(),
                );
                Ok(())
            }
        },
    };
    if let Err(err) = result {
        println!("{}", err)
    }
}
