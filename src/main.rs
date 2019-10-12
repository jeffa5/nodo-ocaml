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

use cli::{Cli, NodoOpts, SubCommand};
use commands::Command;
use commands::{Edit, Format, List, New, Overview, Remove};
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
        None => res = Overview::exec(config, NodoOpts::default()),
        Some(sub_command) => match sub_command {
            SubCommand::New(cli::New { nodo_opts }) => res = New::exec(config, nodo_opts),
            SubCommand::List(cli::List { nodo_opts }) => res = List::exec(config, nodo_opts),
            SubCommand::Remove(cli::Remove { nodo_opts }) => res = Remove::exec(config, nodo_opts),
            SubCommand::Edit(cli::Edit { nodo_opts }) => res = Edit::exec(config, nodo_opts),
            SubCommand::Format(cli::Format { nodo_opts }) => res = Format::exec(config, nodo_opts),
            SubCommand::Overview(cli::Overview { nodo_opts }) => {
                res = Overview::exec(config, nodo_opts)
            }
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
