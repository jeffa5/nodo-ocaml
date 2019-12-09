use chrono::NaiveDate;
use std::cmp::Ordering;
/// A Nodo is a mixture of a todo and a note.
///
/// They are formed of optional metadata and blocks.

/// A block represents a block-like element in the document
///
/// They are rather simple constructs like lists and headings etc.
/// They help to split up the document for other operations within Nodo.
#[derive(Debug, PartialEq, Clone)]
pub enum Block {
    /// A heading text with a level
    Heading(Text, u32),
    /// A sequence of elements, elements can be text or tasks, the list is either numbered or not
    List(List),
    /// A sequence of lines of text
    Paragraph(Vec<Text>),
    /// A separator in the text, used to visually separate blocks
    Rule,
    /// A block quoted sequence of blocks
    BlockQuote(Vec<Block>),
    /// A code block with language
    Code(String, Vec<String>),
}

/// A single line of potentially decorated text
#[derive(Debug, PartialEq, Default, Clone)]
pub struct Text {
    pub inner: Vec<TextItem>,
}

impl From<Vec<TextItem>> for Text {
    fn from(vec: Vec<TextItem>) -> Self {
        Self { inner: vec }
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for item in self.inner.iter() {
            match item {
                TextItem::PlainText(s) => write!(f, "{}", s)?,
                TextItem::StyledText(s, _) => write!(f, "{}", s)?,
                TextItem::Link(s, _) => write!(f, "{}", s)?,
            }
        }
        Ok(())
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

#[derive(Debug, PartialEq, Clone)]
pub enum List {
    Plain(Vec<ListItem>),
    Numbered(Vec<ListItem>, u32),
}

/// A list item is a possible item in a list
#[derive(Debug, PartialEq, Clone)]
pub enum ListItem {
    /// Texts have text and optionally a sublist associated with them
    Text(Vec<Block>, Option<List>),
    /// Tasks have text, completion status and optionally a sublist associated with them
    Task(Vec<Block>, bool, Option<List>),
}

/// Nodos have explicit fields for their metadata e.g. title and tags
/// Other content within the nodo is represented as a sequence of Blocks
#[derive(Debug, PartialEq, Default)]
pub struct Nodo {
    tags: Vec<String>,
    start_date: Option<NaiveDate>,
    due_date: Option<NaiveDate>,
    title: Text,
    /// The rest of the content
    blocks: Vec<Block>,
}

impl Nodo {
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn start_date(&self) -> Option<NaiveDate> {
        self.start_date
    }

    pub fn due_date(&self) -> Option<NaiveDate> {
        self.due_date
    }

    pub fn title(&self) -> &Text {
        &self.title
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn sort_tasks(&mut self) {
        for b in self.blocks.iter_mut() {
            if let Block::List(l) = b {
                // Don't want to sort numbered lists
                if let List::Plain(items) = l {
                    items.sort_by(|a, b| match a {
                        ListItem::Text(_, _) => Ordering::Equal,
                        ListItem::Task(_, c, _) => match b {
                            ListItem::Text(_, _) => Ordering::Equal,
                            ListItem::Task(_, d, _) => {
                                if !c && !d {
                                    Ordering::Equal
                                } else if !c && *d {
                                    Ordering::Less
                                } else if *c && !d {
                                    Ordering::Greater
                                } else {
                                    Ordering::Equal
                                }
                            }
                        },
                    })
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct NodoBuilder {
    nodo: Nodo,
}

impl NodoBuilder {
    pub fn build(self) -> Nodo {
        self.nodo
    }

    pub fn tags(&mut self, tags: Vec<String>) -> &mut Self {
        self.nodo.tags = tags;
        self
    }

    pub fn start_date(&mut self, start_date: NaiveDate) -> &mut Self {
        self.nodo.start_date = Some(start_date);
        self
    }

    pub fn due_date(&mut self, due_date: NaiveDate) -> &mut Self {
        self.nodo.due_date = Some(due_date);
        self
    }

    pub fn title(&mut self, title: Text) -> &mut Self {
        self.nodo.title = title;
        self
    }

    pub fn block(&mut self, block: Block) -> &mut Self {
        self.nodo.blocks.push(block);
        self
    }
}
