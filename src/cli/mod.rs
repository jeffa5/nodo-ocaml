use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "nodo", about = "A task and notes tracker, combined.")]
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

    #[structopt(about = "Generates completion scripts for your shell")]
    Completions {
        #[structopt(possible_values = &structopt::clap::Shell::variants(), about="The shell to generate the script for")]
        shell: String,
    },
}

#[derive(Debug, StructOpt)]
pub struct NodoOpts {
    /// A list of tags for the nodo, separated by ','
    #[structopt(short, long, use_delimiter = true, require_delimiter = true)]
    pub tags: Vec<String>,

    /// A '/' separated value of the form project/subproject/.../nodo_name
    #[structopt(use_delimiter = true, value_delimiter = "/", require_delimiter = true)]
    pub target: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub struct New {
    #[structopt(flatten)]
    pub nodo_opts: NodoOpts,
}

#[derive(Debug, StructOpt)]
pub struct List {
    #[structopt(flatten)]
    pub nodo_opts: NodoOpts,
}

#[derive(Debug, StructOpt)]
pub struct Remove {
    #[structopt(flatten)]
    pub nodo_opts: NodoOpts,
}

#[derive(Debug, StructOpt)]
pub struct Edit {
    #[structopt(flatten)]
    pub nodo_opts: NodoOpts,
}
