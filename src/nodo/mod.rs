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
    Heading(Text, u32),
    /// A sequence of elements, elements can be text or tasks
    List(Vec<ListItem>),
    /// A sequence of lines of text
    Paragraph(Vec<Text>),
    /// A separator in the text, used to visually separate blocks
    Rule,
    /// A block quoted sequence of blocks
    BlockQuote(Vec<Block>),
    /// A code block
    Code(Vec<String>),
}

/// A single line of potentially decorated text
#[derive(Debug, PartialEq)]
pub struct Text {
    pub inner: Vec<TextItem>,
}

impl From<Vec<TextItem>> for Text {
    fn from(vec: Vec<TextItem>) -> Self {
        Self { inner: vec }
    }
}

/// A piece of text can be decorated with a style
#[derive(Debug, PartialEq, Clone)]
pub enum TextItem {
    /// Just text
    PlainText(String),
    /// A piece of styled text
    StyledText(String, TextStyle),
    /// A link, the first field is the name and the second the uri
    Link(String, String),
}

impl TextItem {
    pub fn emphasis(text: &str) -> Self {
        Self::StyledText(text.to_string(), TextStyle::Emphasis)
    }

    pub fn strong(text: &str) -> Self {
        Self::StyledText(text.to_string(), TextStyle::Strong)
    }

    pub fn strikethrough(text: &str) -> Self {
        Self::StyledText(text.to_string(), TextStyle::Strikethrough)
    }

    pub fn code(text: &str) -> Self {
        Self::StyledText(text.to_string(), TextStyle::Code)
    }

    pub fn plain(text: &str) -> Self {
        Self::PlainText(text.to_string())
    }

    pub fn link(name: &str, uri: &str) -> Self {
        Self::Link(name.to_string(), uri.to_string())
    }
}

/// A style for a piece of text
#[derive(Debug, PartialEq, Clone)]
pub enum TextStyle {
    Emphasis,
    Strong,
    Strikethrough,
    Code,
}

/// A list item is a possible item in a list
#[derive(Debug, PartialEq)]
pub enum ListItem {
    /// Tasks have text, completion status and optionally a sublist associated with them
    Task(Text, bool, Option<Vec<ListItem>>),
    /// Texts have text and optionally a sublist associated with them
    Text(Text, Option<Vec<ListItem>>),
}

/// Metadata stores information about the nodo
#[derive(Debug, PartialEq)]
pub struct Metadata {
    projects: Vec<String>,
    tags: Vec<String>,
    title: Text,
    target: String,
}

impl Metadata {
    fn new() -> Self {
        Self {
            projects: Vec::new(),
            tags: Vec::new(),
            title: Text { inner: Vec::new() },
            target: String::new(),
        }
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn projects(&self) -> &[String] {
        &self.projects
    }

    pub fn title(&self) -> &Text {
        &self.title
    }

    pub fn target(&self) -> &str {
        &self.target
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

    pub fn title(mut self, title: Text) -> Self {
        self.metadata.title = title;
        self
    }

    pub fn target(mut self, target: String) -> Self {
        self.metadata.target = target;
        self
    }

    pub fn tags(mut self, tags: &[String]) -> Self {
        self.metadata.tags = tags.to_vec();
        self
    }

    pub fn heading(mut self, text: Text, level: u32) -> Self {
        self.blocks.push(Block::Heading(text, level));
        self
    }

    pub fn list(mut self, items: Vec<ListItem>) -> Self {
        self.blocks.push(Block::List(items));
        self
    }

    pub fn paragraph(mut self, lines: Vec<Text>) -> Self {
        self.blocks.push(Block::Paragraph(lines));
        self
    }

    pub fn rule(mut self) -> Self {
        self.blocks.push(Block::Rule);
        self
    }

    pub fn blockquote(mut self, lines: Vec<Block>) -> Self {
        self.blocks.push(Block::BlockQuote(lines));
        self
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
}
