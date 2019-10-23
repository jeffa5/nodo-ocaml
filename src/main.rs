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
    if opts.sub_command.is_some() && !opts.target.is_empty() {
        println!("Can't specify a target here and a subcommand");
        return;
    }
    match opts.sub_command {
        None => {
            if !config.root_dir.join(&opts.target.inner).exists() {
                Cli::clap().print_help().unwrap();
                return;
            }
            let overview = cli::Overview {
                target: opts.target,
            };
            if let Err(err) = overview.exec(config) {
                println!("{}", err)
            }
        }
        Some(sub_command) => match sub_command {
            SubCommand::New(new) => {
                if let Err(err) = new.exec(config) {
                    println!("{}", err)
                }
            }
            SubCommand::Show(show) => {
                if let Err(err) = show.exec(config) {
                    println!("{}", err)
                }
            }
            SubCommand::Remove(remove) => {
                if let Err(err) = remove.exec(config) {
                    println!("{}", err)
                }
            }
            SubCommand::Edit(edit) => {
                if let Err(err) = edit.exec(config) {
                    println!("{}", err)
                }
            }
            SubCommand::Clean(clean) => {
                if let Err(err) = clean.exec(config) {
                    println!("{}", err)
                }
            }
            SubCommand::Format(format) => {
                if let Err(err) = format.exec(config) {
                    println!("{}", err)
                }
            }
            SubCommand::Overview(overview) => {
                if let Err(err) = overview.exec(config) {
                    println!("{}", err)
                }
            }
            SubCommand::Archive(archive) => {
                if let Err(err) = archive.exec(config) {
                    println!("{}", err)
                }
            }
            SubCommand::Completions { shell } => Cli::clap().gen_completions_to(
                "nodo",
                shell
                    .parse()
                    .expect("Failed to parse shell as a shell candidate"),
                &mut std::io::stdout(),
            ),
        },
    }
}
