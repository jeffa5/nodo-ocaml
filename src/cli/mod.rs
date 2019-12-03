use std::ops;
use std::path::PathBuf;
use structopt::StructOpt;

/// A task and notes tracker, combined
#[derive(StructOpt, Debug)]
#[structopt(name = "nodo")]
pub struct Cli {
    #[structopt(subcommand)]
    pub sub_command: Option<SubCommand>,
    /// Suppress logging completely, overrides '-l'
    #[structopt(short, long, global = true)]
    pub quiet: bool,
    /// Log with the given verbosity, more 'l' is more verbose
    #[structopt(short, long, parse(from_occurrences), global = true)]
    pub log_level: usize,
    /// Set the filetype to use for this command
    #[structopt(long, global = true)]
    pub filetype: Option<String>,
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

    /// Archive a nodo or project tree
    Archive(Archive),
}

#[derive(Debug, Default, StructOpt, PartialEq, Clone)]
pub struct Target {
    /// A '/' separated value of the form project/subproject/.../nodo_name
    #[structopt(name = "target", default_value = "", hide_default_value(true))]
    pub inner: String,
}

impl ops::Deref for Target {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(StructOpt, Debug, Default, PartialEq, Clone)]
pub struct Template {
    /// Create the nodo from the given template file
    #[structopt(name = "template", long, parse(from_os_str))]
    pub inner: Option<PathBuf>,
}

impl ops::Deref for Template {
    type Target = Option<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, StructOpt)]
pub struct New {
    #[structopt(flatten)]
    pub target: Target,

    #[structopt(flatten)]
    pub template: Template,
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
    pub complete: Option<bool>,
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

    #[structopt(flatten)]
    pub template: Template,

    /// Use a temporary file, use `nodo clean` to tidy up
    #[structopt(short, long)]
    pub temp: bool,

    /// Create the file if it doesn't already exist
    #[structopt(short, long)]
    pub create: bool,
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
pub struct Archive {
    #[structopt(flatten)]
    pub target: Target,
}
