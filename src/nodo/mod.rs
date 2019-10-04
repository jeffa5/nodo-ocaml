/// A Nodo is a mixture of a todo and a note.
///
/// They are formed of optional frontmatter which details the metadata of the Nodo

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

/// Nodos have explicit fields for their metadata e.g. project and tags
/// Other content within the nodo is represented as a sequence of Blocks
#[derive(Debug, PartialEq)]
pub struct Nodo {
    /// The project the nodo belongs to
    project: String,
    /// The tags associated with this nodo
    tags: Vec<String>,
    /// The rest of the content
    blocks: Vec<Block>,
}

impl Nodo {
    pub fn new() -> Nodo {
        Nodo {
            project: "".to_string(),
            tags: Vec::new(),
            blocks: Vec::new(),
        }
    }

    pub fn project(&mut self, project: &str) -> &mut Self {
        self.project = project.to_owned();
        self
    }

    pub fn tags(&mut self, tags: &[String]) -> &mut Self {
        self.tags = tags.to_vec();
        self
    }

    pub fn heading(&mut self, level: u32, text: String) -> &mut Self {
        self.blocks.push(Block::Heading(level, text));
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
}
