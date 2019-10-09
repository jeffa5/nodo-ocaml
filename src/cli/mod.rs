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
    #[structopt(visible_aliases=&["create"])]
    New(New),

    /// List available projects and nodos
    #[structopt(visible_aliases=&["ls"])]
    List(List),

    /// Remove a nodo
    #[structopt(visible_aliases=&["rm"])]
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
    /// Projects can be nested using 'project/subproject/subsubproject' syntax
    #[structopt(
        short,
        long,
        use_delimiter = true,
        require_delimiter = true,
        value_delimiter = "/"
    )]
    pub projects: Vec<String>,

    /// A list of tags for the nodo, separated by ','
    #[structopt(short, long, use_delimiter = true, require_delimiter = true)]
    pub tags: Vec<String>,

    /// Filename to use to store this nodo
    pub filename: Option<String>,

    /// The title for this nodo
    pub title: Vec<String>,
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
