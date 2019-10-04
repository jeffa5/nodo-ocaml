/// A Nodo is a mixture of a todo and a note.
///
/// They are formed of optional metadata and blocks.

/// A block represents a block-like element in the document
///
/// They are rather simple constructs like lists and headings etc.
/// They help to split up the document for other operations within Nodo.
#[derive(Debug, PartialEq)]
pub enum Block {
    /// A heading text with a level
    Heading(String, u32),
    /// A sequence of elements, elements must be strings for now
    List(Vec<String>),
    /// A sequence of checkboxes
    Checkboxes(Vec<Checkbox>),
}

/// A checkbox represents a todo item which can be completed or not
#[derive(Debug, PartialEq, Clone)]
pub struct Checkbox {
    text: String,
    checked: bool,
}

impl Checkbox {
    pub fn new(text: String, checked: bool) -> Checkbox {
        Checkbox { text, checked }
    }
}

/// Metadata stores information about the nodo
#[derive(Debug, PartialEq)]
pub struct Metadata {
    projects: Vec<String>,
    tags: Vec<String>,
}

impl Metadata {
    fn new() -> Metadata {
        Metadata {
            projects: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

/// Nodos have explicit fields for their metadata e.g. project and tags
/// Other content within the nodo is represented as a sequence of Blocks
#[derive(Debug, PartialEq)]
pub struct Nodo {
    /// The metadata associated with this nodo
    metadata: Metadata,
    /// The rest of the content
    blocks: Vec<Block>,
}

impl Nodo {
    pub fn new() -> Nodo {
        Nodo {
            metadata: Metadata::new(),
            blocks: Vec::new(),
        }
    }

    pub fn project(&mut self, project: &str) -> &mut Self {
        self.metadata.projects.push(project.to_owned());
        self
    }

    pub fn tags(&mut self, tags: &[String]) -> &mut Self {
        self.metadata.tags = tags.to_vec();
        self
    }

    pub fn heading(&mut self, text: String, level: u32) -> &mut Self {
        self.blocks.push(Block::Heading(text, level));
        self
    }

    pub fn list(&mut self, items: &[String]) -> &mut Self {
        self.blocks.push(Block::List(items.to_vec()));
        self
    }

    pub fn checkboxes(&mut self, items: &[Checkbox]) -> &mut Self {
        self.blocks.push(Block::Checkboxes(items.to_vec()));
        self
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
}
