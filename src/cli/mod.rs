use std::ops;
use structopt::StructOpt;

/// A task and notes tracker, combined
#[derive(StructOpt, Debug)]
#[structopt(name = "nodo")]
pub struct Cli {
    #[structopt(subcommand)]
    pub sub_command: Option<SubCommand>,

    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: usize,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    /// Create a new nodo
    New(New),

    /// List available projects and nodos
    List(List),

    /// Remove a nodo
    Remove(Remove),

    /// Edit a nodo in the editor
    Edit(Edit),

    /// Format nodos or a target
    Format(Format),

    /// Provide an overview of the target
    Overview(Overview),

    /// Generates completion scripts for your shell
    Completions {
        /// The shell to generate the script for
        #[structopt(possible_values = &structopt::clap::Shell::variants())]
        shell: String,
    },
}

#[derive(Debug, Default, StructOpt)]
pub struct Target {
    /// A '/' separated value of the form project/subproject/.../nodo_name
    #[structopt(use_delimiter = true, value_delimiter = "/", require_delimiter = true)]
    pub target: Vec<String>,
}

impl ops::Deref for Target {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

#[derive(Debug, StructOpt)]
pub struct New {
    #[structopt(flatten)]
    pub target: Target,
}

#[derive(Debug, StructOpt)]
pub struct List {
    #[structopt(flatten)]
    pub target: Target,
}

#[derive(Debug, StructOpt)]
pub struct Remove {
    #[structopt(flatten)]
    pub target: Target,
}

#[derive(Debug, StructOpt)]
pub struct Edit {
    #[structopt(flatten)]
    pub target: Target,
}

#[derive(Debug, StructOpt)]
pub struct Format {
    #[structopt(flatten)]
    pub target: Target,

    /// Don't apply the formatting, instead write the formatted file to stdout
    #[structopt(short, long)]
    pub dry_run: bool,
}

#[derive(Debug, StructOpt)]
pub struct Overview {
    #[structopt(flatten)]
    pub target: Target,
}
