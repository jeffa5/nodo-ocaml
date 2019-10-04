use crate::files::{NodoProvider, NodoReader, ReadError};
use crate::nodo::Nodo;
use pulldown_cmark::{Event, Options, Parser, Tag};

pub struct Markdown {
    nodo: Nodo,
}

impl Markdown {
    pub fn new() -> Markdown {
        Markdown { nodo: Nodo::new() }
    }
}

impl NodoProvider for Markdown {
    fn nodo(&self) -> &Nodo {
        &self.nodo
    }
}

struct EventsIter<'a> {
    events: Vec<Event<'a>>,
    index: usize,
}

impl<'a> Iterator for &mut EventsIter<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.events.len() {
            let item = self.events[self.index].clone();
            self.index += 1;
            println!("{:?}", item);
            return Some(item);
        }
        None
    }
}

impl NodoReader for Markdown {
    fn read<R: std::io::Read>(&mut self, mut reader: R) -> Result<(), ReadError> {
        let mut s = String::new();
        reader.read_to_string(&mut s).unwrap();

        let options = Options::all();
        let parser = Parser::new_ext(&s, options);

        let mut events_iter = EventsIter {
            events: parser.collect(),
            index: 0,
        };

        // parse header
        self.parse_frontmatter(&mut events_iter)?;

        // parse any other events
        self.parse_body(&mut events_iter)?;

        Ok(())
    }
}

impl Markdown {
    fn parse_frontmatter(&mut self, events_iter: &mut EventsIter) -> Result<(), ReadError> {
        let mut in_frontmatter = false;
        for event in events_iter {
            if !in_frontmatter {
                match event {
                    Event::Rule => in_frontmatter = true,
                    _ => return Ok(()),
                }
            } else {
                // in_frontmatter
                match event {
                    Event::Rule | Event::End(Tag::Heading(_)) => return Ok(()),
                    Event::Start(Tag::Heading(_)) | Event::SoftBreak | Event::HardBreak => continue,
                    Event::Text(text) => {
                        let text = text.trim();
                        if text.starts_with("project:") {
                            self.nodo
                                .project(text.trim_start_matches("project:").trim());
                        } else if text.starts_with("tags:") {
                            self.nodo.tags(
                                &text
                                    .trim_start_matches("tags:")
                                    .split(',')
                                    .map(|t| t.trim().to_owned())
                                    .collect::<Vec<_>>(),
                            );
                        }
                    }
                    _ => {
                        return Err(ReadError::InvalidElement(format!(
                            "Invalid element in frontmatter: {:?}",
                            event
                        )))
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_body(&mut self, mut events_iter: &mut EventsIter) -> Result<(), ReadError> {
        while let Some(event) = events_iter.next() {
            match event {
                Event::Start(Tag::Heading(level)) => {
                    self.nodo.heading(read_heading(&mut events_iter), level);
                }
                Event::Start(Tag::List(_first_index)) => {
                    self.nodo.list(&read_list(&mut events_iter));
                }
                _ => continue,
            }
        }
        Ok(())
    }
}

fn read_heading(events_iter: &mut EventsIter) -> String {
    for event in events_iter {
        match event {
            Event::Text(t) => return t.into_string(),
            _ => continue,
        }
    }
    String::new()
}

fn read_list(mut events_iter: &mut EventsIter) -> Vec<String> {
    let mut items = Vec::new();
    while let Some(event) = events_iter.next() {
        match event {
            Event::Start(Tag::Item) => items.push(read_list_item(&mut events_iter)),
            _ => continue,
        }
    }
    items
}

fn read_list_item(events_iter: &mut EventsIter) -> String {
    for event in events_iter {
        match event {
            Event::Text(t) => return t.into_string(),
            _ => continue,
        }
    }
    String::new()
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_md() {
        let f = std::fs::File::open("test/markdown/nodo1.md");
        let mut md = Markdown::new();
        md.read(f.unwrap()).unwrap();
        assert_eq!(
            md.nodo(),
            Nodo::new()
                .project("myproject")
                .tags(&[
                    "nodo".to_owned(),
                    "more tags".to_owned(),
                    "hey another tag".to_owned()
                ])
                .heading("nodo header".to_owned(), 1)
                .list(&["list item 1".to_owned(), "list item 2".to_owned()])
        );
    }
}
