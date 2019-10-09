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
use commands::Command;
use commands::{Edit, List, New, Remove};
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
    let mut nodo = files::get_nodo(config.default_filetype);
    let mut res = Ok(());
    match opts.sub_command {
        None => println!("Something special"),
        Some(sub_command) => match sub_command {
            SubCommand::New(cli::New { nodo_opts }) => {
                nodo = build_nodo(nodo, nodo_opts);
                res = New::exec(config, nodo)
            }
            SubCommand::List(cli::List { nodo_opts }) => {
                nodo = build_nodo(nodo, nodo_opts);
                res = List::exec(config, nodo)
            }
            SubCommand::Remove(cli::Remove { nodo_opts }) => {
                nodo = build_nodo(nodo, nodo_opts);
                res = Remove::exec(config, nodo)
            }
            SubCommand::Edit(cli::Edit { nodo_opts }) => {
                nodo = build_nodo(nodo, nodo_opts);
                res = Edit::exec(config, nodo)
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

fn build_nodo<F: files::NodoFile>(nodo: nodo::Nodo<F>, nodo_opts: cli::NodoOpts) -> nodo::Nodo<F> {
    nodo.filename(nodo_opts.filename.unwrap_or_default())
        .projects(&nodo_opts.projects)
        .tags(&nodo_opts.tags)
        .title(nodo_opts.title.join(" "))
}
