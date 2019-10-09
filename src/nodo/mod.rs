use crate::files::NodoFile;
use std::marker::PhantomData;
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
    /// A sequence of elements, elements can be text or tasks
    List(Vec<ListItem>),
}

/// A checkbox represents a todo item which can be completed or not
#[derive(Debug, PartialEq, Clone)]
pub struct Task {
    pub text: String,
    pub checked: bool,
}

impl Task {
    pub fn new(text: String, checked: bool) -> Self {
        Self { text, checked }
    }
}

#[derive(Debug, PartialEq)]
pub enum ListItem {
    Task(Task),
    Text(String),
}

/// Metadata stores information about the nodo
#[derive(Debug, PartialEq)]
pub struct Metadata {
    projects: Vec<String>,
    tags: Vec<String>,
    title: String,
    filename: String,
}

impl Metadata {
    fn new() -> Self {
        Self {
            projects: Vec::new(),
            tags: Vec::new(),
            title: String::new(),
            filename: String::new(),
        }
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn projects(&self) -> &[String] {
        &self.projects
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
}

/// Nodos have explicit fields for their metadata e.g. project and tags
/// Other content within the nodo is represented as a sequence of Blocks
#[derive(Debug, PartialEq)]
pub struct Nodo<F: NodoFile> {
    filetype: PhantomData<F>,
    /// The metadata associated with this nodo
    metadata: Metadata,
    /// The rest of the content
    blocks: Vec<Block>,
}

impl<F: NodoFile> Nodo<F> {
    pub fn new() -> Nodo<F> {
        Nodo {
            filetype: PhantomData,
            metadata: Metadata::new(),
            blocks: Vec::new(),
        }
    }

    pub fn projects(mut self, projects: &[String]) -> Self {
        self.metadata.projects.append(&mut projects.to_vec());
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.metadata.title = title;
        self
    }

    pub fn filename(mut self, filename: String) -> Self {
        self.metadata.filename = filename;
        self
    }

    pub fn tags(mut self, tags: &[String]) -> Self {
        self.metadata.tags = tags.to_vec();
        self
    }

    pub fn heading(mut self, text: String, level: u32) -> Self {
        self.blocks.push(Block::Heading(text, level));
        self
    }

    pub fn list(mut self, items: Vec<ListItem>) -> Self {
        self.blocks.push(Block::List(items));
        self
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
}
