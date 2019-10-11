use log::*;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::io::{Read, Write};

use crate::files::{NodoFile, ReadError, WriteError};
use crate::nodo::{Block, ListItem, Nodo, Text, TextItem, TextStyle};

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
    const EXTENSION: &'static str = "markdown";

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
        writeln!(writer, "---")?;
        writeln!(writer, "tags: {}", nodo.metadata().tags().join(", "))?;
        writeln!(writer, "---")?;
        writeln!(writer)?;

        // write title as header with level 1
        write_heading(writer, nodo.metadata().title(), 1)?;
        writeln!(writer)?;
        // write fields to the file

        for (i, block) in nodo.blocks().iter().enumerate() {
            match block {
                Block::List(items) => write_list(writer, items, 0)?,
                Block::Heading(t, l) => write_heading(writer, t, *l)?,
                Block::Paragraph(lines) => write_paragraph(writer, lines)?,
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
            Event::Start(Tag::Paragraph) => nodo = nodo.paragraph(read_paragraph(&mut events_iter)),
            e => {
                error!("read body reached unimplemented event: {:?}", e);
                unimplemented!()
            }
        }
    }
    Ok(nodo)
}

fn read_paragraph(mut events_iter: &mut EventsIter) -> Vec<Text> {
    let mut lines = Vec::new();
    let mut line = Vec::new();
    while let Some(event) = events_iter.next() {
        match event {
            Event::End(Tag::Paragraph) => return vec![line.into()],
            Event::Text(t) => line.push(TextItem::PlainText(t.to_string())),
            Event::SoftBreak => {
                lines.push(line);
                line = Vec::new()
            }
            Event::Start(Tag::Emphasis) => line.push(read_text_item(events_iter)),
            Event::Start(Tag::Strong) => line.push(read_text_item(events_iter)),
            Event::Start(Tag::Strikethrough) => line.push(read_text_item(events_iter)),
            Event::Code(string) => line.push(TextItem::code(&string)),
            Event::Start(Tag::Link(_inline, url, _title)) => {
                line.push(read_link(events_iter, &url))
            }
            _ => unimplemented!(),
        }
    }
    Vec::new()
}

fn read_heading(events_iter: &mut EventsIter) -> Text {
    let mut text = Vec::new();
    for event in events_iter {
        match event {
            Event::Text(t) => text.push(TextItem::plain(&t)),
            Event::Start(Tag::Heading(_)) => continue,
            Event::End(Tag::Heading(_)) => return text.into(),
            e => {
                error!("read heading reached unimplemented event: {:?}", e);
                unimplemented!()
            }
        }
    }
    Vec::new().into()
}

fn read_list(mut events_iter: &mut EventsIter) -> Vec<ListItem> {
    let mut items = Vec::new();
    while let Some(event) = events_iter.next() {
        match event {
            Event::Start(Tag::Item) => items.push(read_list_item(&mut events_iter)),
            Event::End(Tag::List(_first_index)) => return items,
            e => {
                error!("read list reached unimplemented event: {:?}", e);
                unimplemented!()
            }
        }
    }
    items
}

fn read_list_item(mut events_iter: &mut EventsIter) -> ListItem {
    let mut text = Vec::new();
    let mut is_task = false;
    let mut completed = false;
    let mut nested_list = None;
    while let Some(event) = events_iter.next() {
        match event {
            Event::Text(t) => text.push(TextItem::plain(&t)),
            Event::End(Tag::Item) => {
                if is_task {
                    return ListItem::Task(text.into(), completed, nested_list);
                } else {
                    // check for [, then "\s", then ], then strip front whitespace of other
                    // FIXME: ugly code, probably a nicer and cleaner way to do it
                    let mut text_iter = text.iter_mut();
                    if let Some(TextItem::PlainText(t)) = text_iter.next() {
                        if t.trim() == "[" {
                            // \s then ]
                            if let Some(TextItem::PlainText(t)) = text_iter.next() {
                                if ["x", "X", ""].iter().any(|x| x == &t.trim()) {
                                    let complete = t.trim() != "";
                                    // ]
                                    if let Some(TextItem::PlainText(t)) = text_iter.next() {
                                        if t.trim() == "]" {
                                            // yay we have a task
                                            if let Some(textitem) = text_iter.next() {
                                                match textitem {
                                                    TextItem::PlainText(t)
                                                    | TextItem::StyledText(t, _) => {
                                                        *t = t.trim_start().to_string()
                                                    }
                                                    _ => unimplemented!(),
                                                }
                                            }
                                            return ListItem::Task(
                                                text[3..].to_vec().into(),
                                                complete,
                                                nested_list,
                                            );
                                        }
                                    }
                                } else if t.trim() == "]" {
                                    if let Some(textitem) = text_iter.next() {
                                        match textitem {
                                            TextItem::PlainText(t) | TextItem::StyledText(t, _) => {
                                                *t = t.trim_start().to_string()
                                            }
                                            _ => unimplemented!(),
                                        }
                                    }
                                    return ListItem::Task(
                                        text[2..].to_vec().into(),
                                        false,
                                        nested_list,
                                    );
                                }
                            }
                        }
                    }
                    return ListItem::Text(text.into(), nested_list);
                }
            }
            Event::Start(Tag::List(_)) => nested_list = Some(read_list(events_iter)),
            Event::TaskListMarker(ticked) => {
                is_task = true;
                completed = ticked;
            }
            Event::Start(Tag::Emphasis) => text.push(read_text_item(events_iter)),
            Event::Start(Tag::Strong) => text.push(read_text_item(events_iter)),
            Event::Start(Tag::Strikethrough) => text.push(read_text_item(events_iter)),
            Event::Code(string) => text.push(TextItem::code(&string)),
            Event::Start(Tag::Link(_inline, url, _title)) => {
                text.push(read_link(events_iter, &url))
            }
            e => {
                error!("read list item reached unimplemented event: {:?}", e);
                unimplemented!()
            }
        }
    }
    ListItem::Text(Vec::new().into(), None)
}

fn read_link(events_iter: &mut EventsIter, uri: &str) -> TextItem {
    let mut name = String::new();
    for event in events_iter {
        match event {
            Event::End(Tag::Link(_, _, _)) => return TextItem::link(&name, uri),
            Event::Text(t) => name += &t,
            _ => unimplemented!(),
        }
    }
    TextItem::Link(String::new(), String::new())
}

fn read_text_item(events_iter: &mut EventsIter) -> TextItem {
    let mut string = String::new();
    for event in events_iter {
        match event {
            Event::Text(s) => string.push_str(&s),
            Event::End(Tag::Emphasis) => return TextItem::emphasis(&string),
            Event::End(Tag::Strong) => return TextItem::strong(&string),
            Event::End(Tag::Strikethrough) => return TextItem::strikethrough(&string),
            e => {
                error!("read text item reached unimplemented event: {:?}", e);
                unimplemented!()
            }
        }
    }
    TextItem::plain("")
}

fn write_paragraph<W: Write>(writer: &mut W, lines: &[Text]) -> Result<(), WriteError> {
    for line in lines {
        writeln!(writer, "{}", format_text(&line))?
    }
    Ok(())
}

fn write_heading<W: Write>(writer: &mut W, text: &Text, level: u32) -> Result<(), WriteError> {
    writeln!(
        writer,
        "{}",
        &format!("{} {}", "#".repeat(level as usize), format_text(text))
    )?;
    Ok(())
}

fn format_text(text: &Text) -> String {
    let mut s = String::new();
    for item in text.inner.iter() {
        if cfg!(test) {
            eprintln!("format_text with text item: {:?}", item);
        }
        match item {
            TextItem::PlainText(t) => s += t,
            TextItem::StyledText(t, style) => match style {
                TextStyle::Strikethrough => s += &format!("~~{}~~", t),
                TextStyle::Strong => s += &format!("**{}**", t),
                TextStyle::Emphasis => s += &format!("*{}*", t),
                TextStyle::Code => s += &format!("`{}`", t),
            },
            TextItem::Link(name, uri) => s += &format!("[{}]({})", name, uri),
        }
    }
    s
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
                writeln!(writer, "{}", &format!("{}- {}", indent, format_text(s)))?;
                match nested_list {
                    Some(nl) => write_list(writer, nl, level + 1)?,
                    None => (),
                }
            }
            ListItem::Task(text, completed, nested_list) => {
                if *completed {
                    writeln!(writer, "{}- [x] {}", indent, format_text(text))?
                } else {
                    writeln!(writer, "{}- [ ] {}", indent, format_text(text))?
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
            .title(vec![TextItem::plain("nodo header level 1, is the title")].into())
            .list(vec![
                ListItem::Text(vec![TextItem::plain("list item 1")].into(), None),
                ListItem::Text(vec![TextItem::plain("list item 2")].into(), None),
            ])
            .heading(vec![TextItem::plain("nodo header with level 2")].into(), 2)
            .list(vec![
                ListItem::Task(
                    vec![TextItem::plain("An item to complete")].into(),
                    false,
                    None,
                ),
                ListItem::Task(
                    vec![
                        TextItem::plain("A "),
                        TextItem::emphasis("completed"),
                        TextItem::plain(" item, yay"),
                    ]
                    .into(),
                    true,
                    Some(vec![
                        ListItem::Task(
                            vec![
                                TextItem::plain("Hey a "),
                                TextItem::strong("nested"),
                                TextItem::plain(" task"),
                            ]
                            .into(),
                            false,
                            None,
                        ),
                        ListItem::Text(
                            vec![
                                TextItem::plain("And a "),
                                TextItem::emphasis("nested"),
                                TextItem::plain(" text"),
                            ]
                            .into(),
                            None,
                        ),
                    ]),
                ),
                ListItem::Text(
                    vec![TextItem::plain("a text list item")].into(),
                    Some(vec![
                        ListItem::Text(
                            vec![
                                TextItem::plain("nested "),
                                TextItem::strong("list"),
                                TextItem::plain(" again"),
                            ]
                            .into(),
                            None,
                        ),
                        ListItem::Task(
                            vec![TextItem::plain("and a "), TextItem::code("task")].into(),
                            false,
                            None,
                        ),
                    ]),
                ),
                ListItem::Task(
                    vec![TextItem::plain("or a ~task~ list item")].into(),
                    false,
                    None,
                ),
                ListItem::Task(
                    vec![
                        TextItem::plain("and a "),
                        TextItem::strikethrough("technically"),
                        TextItem::plain(" ill-formed task, but should be allowed really"),
                    ]
                    .into(),
                    false,
                    None,
                ),
            ])
    }

    static TEST_NODO_UNFORMATTED: &str = "---
tags: nodo, more tags, hey another tag
---

# nodo header level 1, is the title

- list item 1
- list item 2

## nodo header with level 2

- [   ] An item to complete
- [  x       ]     A *completed* item, yay
    - [ ] Hey a **nested** task
    - And a *nested* text
- a text list item
    -    nested **list** again
    - [ ] and a `task`
- [     ]      or a ~task~ list item
- [] and a ~~technically~~ ill-formed task, but should be allowed really
";

    static TEST_NODO_FORMATTED: &str = "---
tags: nodo, more tags, hey another tag
---

# nodo header level 1, is the title

- list item 1
- list item 2

## nodo header with level 2

- [ ] An item to complete
- [x] A *completed* item, yay
    - [ ] Hey a **nested** task
    - And a *nested* text
- a text list item
    - nested **list** again
    - [ ] and a `task`
- [ ] or a ~task~ list item
- [ ] and a ~~technically~~ ill-formed task, but should be allowed really
";

    #[test]
    fn test_formatted_and_unformatted_should_give_same_nodo() {
        assert_eq!(
            Markdown::read(Nodo::new(), &mut TEST_NODO_FORMATTED.as_bytes()).unwrap(),
            Markdown::read(Nodo::new(), &mut TEST_NODO_UNFORMATTED.as_bytes()).unwrap()
        )
    }

    #[test]
    fn test_read() {
        assert_eq!(
            Markdown::read(Nodo::new(), &mut TEST_NODO_FORMATTED.as_bytes()).unwrap(),
            get_test_nodo(),
        );
        assert_eq!(
            Markdown::read(Nodo::new(), &mut TEST_NODO_UNFORMATTED.as_bytes()).unwrap(),
            get_test_nodo(),
        );
    }

    #[test]
    fn test_write() {
        let mut writer: Vec<u8> = Vec::new();
        Markdown::write(&get_test_nodo(), &mut writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            TEST_NODO_FORMATTED.to_owned()
        );
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

    #[test]
    fn test_read_write_gives_same_output() {
        let mut nodo = Nodo::new();
        nodo = Markdown::read(nodo, &mut TEST_NODO_FORMATTED.as_bytes()).unwrap();
        let mut s = Vec::new();

        Markdown::write(&nodo, &mut s).unwrap();
        assert_eq!(String::from_utf8(s).unwrap(), TEST_NODO_FORMATTED);
    }

    #[test]
    fn test_read_without_frontmatter() {
        let s = "# title";
        assert_eq!(
            Markdown::read(Nodo::new(), &mut s.as_bytes()).unwrap(),
            Nodo::new().title(vec![TextItem::plain("title")].into())
        )
    }
}
