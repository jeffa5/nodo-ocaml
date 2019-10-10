use pulldown_cmark::{Event, Options, Parser, Tag};
use std::io::{Read, Write};

use crate::files::{NodoFile, ReadError, WriteError};
use crate::nodo::{Block, ListItem, Nodo};

#[derive(Debug, PartialEq)]
pub struct Markdown;

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
            if cfg!(test) {
                eprintln!("Reading nodo element: {:?}", item);
            }
            return Some(item);
        }
        None
    }
}

impl NodoFile for Markdown {
    const EXTENSION: &'static str = "md";

    fn read<R: Read>(mut nodo: Nodo<Self>, reader: &mut R) -> Result<Nodo<Self>, ReadError> {
        let mut s = String::new();
        reader.read_to_string(&mut s)?;

        let options = Options::all();
        let parser = Parser::new_ext(&s, options);

        let mut events_iter = EventsIter {
            events: parser.collect(),
            index: 0,
        };

        nodo = read_frontmatter(nodo, &mut events_iter)?;

        nodo = nodo.title(read_heading(&mut events_iter));

        nodo = read_body(nodo, &mut events_iter)?;

        Ok(nodo)
    }

    fn write<W: Write>(nodo: &Nodo<Self>, writer: &mut W) -> Result<(), WriteError> {
        let mut metadata_lines = Vec::new();
        let tag_string = nodo.metadata().tags().join(", ");
        if tag_string != "" {
            metadata_lines.push(format!("tags: {}", tag_string))
        }
        let metadata_str = metadata_lines.join("\n");
        if metadata_str != "" {
            writeln!(writer, "---")?;
            writeln!(writer, "{}", &metadata_str)?;
            writeln!(writer, "---\n")?;
        }

        // write title as header with level 1

        write_heading(writer, nodo.metadata().title(), 1)?;
        writeln!(writer)?;
        // write fields to the file

        for (i, block) in nodo.blocks().iter().enumerate() {
            match block {
                Block::List(items) => write_list(writer, items, 0)?,
                Block::Heading(t, l) => write_heading(writer, t, *l)?,
            }
            if i != nodo.blocks().len() - 1 {
                writeln!(writer)?;
            }
        }
        Ok(())
    }
}

fn read_frontmatter<F: NodoFile>(
    mut nodo: Nodo<F>,
    events_iter: &mut EventsIter,
) -> Result<Nodo<F>, ReadError> {
    let mut in_frontmatter = false;
    for event in events_iter {
        if !in_frontmatter {
            match event {
                Event::Rule => in_frontmatter = true,
                _ => return Ok(nodo),
            }
        } else {
            // in_frontmatter
            match event {
                Event::Rule | Event::End(Tag::Heading(_)) => return Ok(nodo),
                Event::Start(Tag::Heading(_)) | Event::SoftBreak | Event::HardBreak => continue,
                Event::Text(text) => {
                    let text = text.trim();
                    if text.starts_with("tags:") {
                        nodo = nodo.tags(
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
    Ok(nodo)
}

fn read_body<F: NodoFile>(
    mut nodo: Nodo<F>,
    mut events_iter: &mut EventsIter,
) -> Result<Nodo<F>, ReadError> {
    while let Some(event) = events_iter.next() {
        match event {
            Event::Start(Tag::Heading(level)) => {
                nodo = nodo.heading(read_heading(&mut events_iter), level);
            }
            Event::Start(Tag::List(_first_index)) => nodo = nodo.list(read_list(&mut events_iter)),
            _ => unreachable!(),
        }
    }
    Ok(nodo)
}

fn read_heading(events_iter: &mut EventsIter) -> String {
    for event in events_iter {
        match event {
            Event::Text(t) => return t.into_string(),
            _ => unreachable!(),
        }
    }
    String::new()
}

fn read_list(mut events_iter: &mut EventsIter) -> Vec<ListItem> {
    let mut items = Vec::new();
    while let Some(event) = events_iter.next() {
        match event {
            Event::Start(Tag::Item) => items.push(read_list_item(&mut events_iter)),
            Event::End(Tag::List(_first_index)) => return items,
            _ => unreachable!(),
        }
    }
    items
}

fn read_list_item(mut events_iter: &mut EventsIter) -> ListItem {
    let mut text = String::new();
    let mut is_task = false;
    let mut completed = false;
    let mut nested_list = None;
    while let Some(event) = events_iter.next() {
        match event {
            Event::Text(t) => text += &t.into_string(),
            Event::End(Tag::Item) => {
                if is_task {
                    return ListItem::Task(text, completed, nested_list);
                } else {
                    return ListItem::Text(text, nested_list);
                }
            }
            Event::Start(Tag::List(_)) => nested_list = Some(read_list(events_iter)),
            Event::TaskListMarker(ticked) => {
                is_task = true;
                completed = ticked;
            }
            _ => unreachable!(),
        }
    }
    ListItem::Text(String::new(), None)
}

fn write_heading<W: Write>(writer: &mut W, text: &str, level: u32) -> Result<(), WriteError> {
    writeln!(
        writer,
        "{}",
        &format!("{} {}", "#".repeat(level as usize), text)
    )?;
    Ok(())
}

fn write_list<W: Write>(
    writer: &mut W,
    list_items: &[ListItem],
    level: usize,
) -> Result<(), WriteError> {
    let indent = "    ".repeat(level);
    for item in list_items {
        match item {
            ListItem::Text(s, nested_list) => {
                writeln!(writer, "{}", &format!("{}- {}", indent, s))?;
                match nested_list {
                    Some(nl) => write_list(writer, nl, level + 1)?,
                    None => (),
                }
            }
            ListItem::Task(text, completed, nested_list) => {
                if *completed {
                    writeln!(writer, "{}- [x] {}", indent, text)?
                } else {
                    writeln!(writer, "{}- [ ] {}", indent, text)?
                }
                match nested_list {
                    Some(nl) => write_list(writer, nl, level + 1)?,
                    None => (),
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    fn get_test_nodo() -> Nodo<Markdown> {
        Nodo::new()
            .tags(&[
                "nodo".to_owned(),
                "more tags".to_owned(),
                "hey another tag".to_owned(),
            ])
            .title("nodo header level 1, is the title".to_owned())
            .list(vec![
                ListItem::Text("list item 1".to_owned(), None),
                ListItem::Text("list item 2".to_owned(), None),
            ])
            .heading("nodo header with level 2".to_owned(), 2)
            .list(vec![
                ListItem::Task("An item to complete".to_string(), false, None),
                ListItem::Task(
                    "A completed item, yay".to_string(),
                    true,
                    Some(vec![
                        ListItem::Task("Hey a nested task".to_owned(), false, None),
                        ListItem::Text("And a nested text".to_owned(), None),
                    ]),
                ),
                ListItem::Text(
                    "a text list item".to_owned(),
                    Some(vec![
                        ListItem::Text("nested list again".to_owned(), None),
                        ListItem::Task("and a task".to_owned(), false, None),
                    ]),
                ),
                ListItem::Task("or a task list item".to_string(), false, None),
            ])
    }

    static TEST_NODO: &str = "---
tags: nodo, more tags, hey another tag
---

# nodo header level 1, is the title

- list item 1
- list item 2

## nodo header with level 2

- [ ] An item to complete
- [x] A completed item, yay
    - [ ] Hey a nested task
    - And a nested text
- a text list item
    - nested list again
    - [ ] and a task
- [ ] or a task list item
";

    #[test]
    fn test_read() {
        assert_eq!(
            Markdown::read(Nodo::new(), &mut TEST_NODO.as_bytes()).unwrap(),
            get_test_nodo(),
        );
    }

    #[test]
    fn test_write() {
        let mut writer: Vec<u8> = Vec::new();
        Markdown::write(&get_test_nodo(), &mut writer).unwrap();
        assert_eq!(String::from_utf8(writer).unwrap(), TEST_NODO.to_owned());
    }

    #[test]
    fn test_write_read_gives_same_nodo() {
        let mut s = Vec::new();
        Markdown::write(&get_test_nodo(), &mut s).unwrap();
        assert_eq!(
            Markdown::read(Nodo::new(), &mut &s[..]).unwrap(),
            get_test_nodo()
        );
    }
}
