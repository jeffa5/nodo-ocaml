use std::ops;
use std::path::PathBuf;
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

    /// Show available projects and nodos
    Show(Show),

    /// Remove a nodo
    Remove(Remove),

    /// Edit a nodo in the editor
    Edit(Edit),

    /// Clean up the temporary directory
    Clean(Clean),

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

#[derive(Debug, Default, StructOpt, PartialEq)]
pub struct Target {
    /// A '/' separated value of the form project/subproject/.../nodo_name
    #[structopt(default_value = "")]
    pub target: String,
}

impl ops::Deref for Target {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.target)
    }
}

#[derive(Debug, StructOpt)]
pub struct New {
    #[structopt(flatten)]
    pub target: Target,

    /// Create the nodo from the given template file
    #[structopt(short, long, parse(from_os_str))]
    pub template: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct Show {
    #[structopt(flatten)]
    pub target: Target,
    /// The max depth to show for a list or the tree
    #[structopt(short, long)]
    pub depth: Option<u32>,
    /// Filter task lists to show only complete tasks
    #[structopt(short, long)]
    pub filter_complete: Option<bool>,
}

#[derive(Debug, StructOpt)]
pub struct Remove {
    #[structopt(flatten)]
    pub target: Target,
    /// Force removal of the target, allows removal of projects
    #[structopt(short, long)]
    pub force: bool,
}

#[derive(Debug, StructOpt)]
pub struct Edit {
    #[structopt(flatten)]
    pub target: Target,

    #[structopt(short, long)]
    pub temp: bool,
}

#[derive(Debug, StructOpt)]
pub struct Clean {}

#[derive(Debug, StructOpt)]
pub struct Format {
    #[structopt(flatten)]
    pub target: Target,

    /// Don't apply the formatting, instead write the formatted file to stdout
    #[structopt(short, long)]
    pub dry_run: bool,

    /// Show output of files being formatted
    #[structopt(short, long)]
    pub verbose: bool,
}

#[derive(Debug, StructOpt)]
pub struct Overview {
    #[structopt(flatten)]
    pub target: Target,
}
