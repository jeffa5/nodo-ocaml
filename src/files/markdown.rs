use chrono::NaiveDate;
use log::*;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::convert::TryInto;
use std::io::{Read, Write};

use crate::config::Config;
use crate::files::{NodoFile, ReadError, WriteError};
use crate::nodo::{Block, List, ListItem, Nodo, NodoBuilder, Text, TextItem, TextStyle};

#[derive(Debug, PartialEq, Default)]
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

    fn read<R: Read>(
        &self,
        mut nodo: NodoBuilder,
        reader: &mut R,
        config: &Config,
    ) -> Result<Nodo, ReadError> {
        let mut s = String::new();
        reader.read_to_string(&mut s)?;

        let options = Options::all();
        let parser = Parser::new_ext(&s, options);

        let mut events_iter = EventsIter {
            events: parser.collect(),
            index: 0,
        };

        read_frontmatter(&mut nodo, &mut events_iter, config)?;

        nodo.title(read_heading(&mut events_iter));

        read_body(&mut nodo, &mut events_iter)?;

        let nodo = nodo.build();
        Ok(nodo)
    }

    fn write<W: Write>(
        &self,
        nodo: &Nodo,
        writer: &mut W,
        config: &Config,
    ) -> Result<(), WriteError> {
        write_frontmatter(nodo, writer, config)?;
        // write title as header with level 1
        write_heading(writer, "", false, nodo.title(), 1)?;
        writeln!(writer)?;
        // write fields to the file

        for (i, block) in nodo.blocks().iter().enumerate() {
            write_block(writer, "", false, block)?;
            if i != nodo.blocks().len() - 1 {
                writeln!(writer)?;
            }
        }
        Ok(())
    }
}

fn write_frontmatter<W: Write>(
    nodo: &Nodo,
    writer: &mut W,
    config: &Config,
) -> Result<(), WriteError> {
    let mut lines = Vec::new();
    debug!("{:?}", nodo.tags());
    if !nodo.tags().is_empty() {
        lines.push(format!("tags: {}", nodo.tags().join(", ")))
    }
    if let Some(start_date) = nodo.start_date() {
        lines.push(format!(
            "start_date: {}",
            start_date.format(config.date_format)
        ))
    }
    if let Some(due_date) = nodo.due_date() {
        lines.push(format!("due_date: {}", due_date.format(config.date_format)))
    }
    if !lines.is_empty() {
        writeln!(writer, "---")?;
        for line in lines {
            writeln!(writer, "{}", line)?
        }
        writeln!(writer, "---")?;
        writeln!(writer)?;
    }
    Ok(())
}

fn write_block<W: Write>(
    writer: &mut W,
    prefix: &str,
    prefix_first_line: bool,
    block: &Block,
) -> Result<(), WriteError> {
    match block {
        Block::List(list) => write_list(writer, prefix, &list)?,
        Block::Heading(t, l) => write_heading(writer, prefix, prefix_first_line, &t, *l)?,
        Block::Paragraph(lines) => write_paragraph(writer, prefix, prefix_first_line, lines)?,
        Block::Rule => writeln!(writer, "{}---", prefix)?,
        Block::BlockQuote(blocks) => write_blockquote(writer, prefix, blocks)?,
        Block::Code(lang, lines) => write_code(writer, prefix, prefix_first_line, lang, lines)?,
    }
    Ok(())
}

fn read_frontmatter(
    nodo: &mut NodoBuilder,
    events_iter: &mut EventsIter,
    config: &Config,
) -> Result<(), ReadError> {
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
                    if text.starts_with("tags:") {
                        let tags = text
                            .trim_start_matches("tags:")
                            .split(',')
                            .map(|t| t.trim().to_string())
                            .collect::<Vec<_>>();
                        if !tags.is_empty() && tags.first().unwrap() != "" {
                            nodo.tags(tags);
                        }
                    } else if text.starts_with("start_date:") {
                        let date = NaiveDate::parse_from_str(
                            text.trim_start_matches("start_date:"),
                            config.date_format,
                        );
                        if let Ok(date) = date {
                            nodo.start_date(date);
                        }
                    } else if text.starts_with("due_date:") {
                        let date = NaiveDate::parse_from_str(
                            text.trim_start_matches("due_date:"),
                            config.date_format,
                        );
                        if let Ok(date) = date {
                            nodo.due_date(date);
                        }
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

fn read_body(nodo: &mut NodoBuilder, mut events_iter: &mut EventsIter) -> Result<(), ReadError> {
    while let Some(event) = events_iter.next() {
        match event {
            Event::Start(Tag::Heading(level)) => {
                nodo.block(Block::Heading(read_heading(&mut events_iter), level));
            }
            Event::Start(Tag::List(first_index)) => match first_index {
                None => {
                    nodo.block(Block::List(List::Plain(read_list(&mut events_iter))));
                }
                Some(index) => {
                    nodo.block(Block::List(List::Numbered(
                        read_list(&mut events_iter),
                        index.try_into().unwrap(),
                    )));
                }
            },
            Event::Start(Tag::Paragraph) => {
                nodo.block(Block::Paragraph(read_paragraph(&mut events_iter)));
            }
            Event::Rule => {
                nodo.block(Block::Rule);
            }
            Event::Start(Tag::BlockQuote) => {
                nodo.block(Block::BlockQuote(read_blockquote(&mut events_iter)));
            }
            Event::Start(Tag::CodeBlock(lang)) => {
                nodo.block(Block::Code(
                    lang.to_string(),
                    read_codeblock(&mut events_iter),
                ));
            }
            e => {
                error!("read body reached unimplemented event: {:?}", e);
                unimplemented!()
            }
        }
    }
    Ok(())
}

fn read_codeblock(events_iter: &mut EventsIter) -> Vec<String> {
    let mut text = Vec::new();
    for event in events_iter {
        match event {
            Event::End(Tag::CodeBlock(_)) => return text,
            Event::Text(t) => text.push(t.to_string()),
            _ => unimplemented!(),
        }
    }
    Vec::new()
}

fn read_blockquote(mut events_iter: &mut EventsIter) -> Vec<Block> {
    let mut blocks = Vec::new();
    while let Some(event) = events_iter.next() {
        match event {
            Event::End(Tag::BlockQuote) => return blocks,
            Event::Start(Tag::Paragraph) => {
                blocks.push(Block::Paragraph(read_paragraph(&mut events_iter)))
            }
            Event::Start(Tag::BlockQuote) => {
                blocks.push(Block::BlockQuote(read_blockquote(events_iter)))
            }
            Event::Start(Tag::Heading(level)) => {
                blocks.push(Block::Heading(read_heading(events_iter), level))
            }
            Event::Start(Tag::List(first_index)) => match first_index {
                None => {
                    blocks.push(Block::List(List::Plain(read_list(&mut events_iter))));
                }
                Some(index) => {
                    blocks.push(Block::List(List::Numbered(
                        read_list(&mut events_iter),
                        index.try_into().unwrap(),
                    )));
                }
            },
            Event::Start(Tag::CodeBlock(language)) => {
                blocks.push(Block::Code(language.to_string(), read_code(events_iter)))
            }
            _ => unimplemented!(),
        }
    }
    Vec::new()
}

fn read_code(events_iter: &mut EventsIter) -> Vec<String> {
    let mut lines = Vec::new();
    for event in events_iter {
        match event {
            Event::End(Tag::CodeBlock(_language)) => return lines,
            Event::Text(t) => lines.push(t.to_string()),
            _ => unimplemented!(),
        }
    }
    Vec::new()
}

fn read_paragraph(mut events_iter: &mut EventsIter) -> Vec<Text> {
    let mut lines = Vec::new();
    let mut line = Vec::new();
    while let Some(event) = events_iter.next() {
        match event {
            Event::End(Tag::Paragraph) => {
                if !line.is_empty() {
                    lines.push(line.into())
                }
                return lines;
            }
            Event::Text(t) => line.push(TextItem::PlainText(t.to_string())),
            Event::SoftBreak => {
                if !line.is_empty() {
                    lines.push(line.into())
                }
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
    let mut blocks = Vec::new();
    let mut lines = Vec::new();
    let mut line = Vec::new();
    let mut is_task = false;
    let mut completed = false;
    let mut nested_list = None;
    while let Some(event) = events_iter.next() {
        match event {
            Event::Text(t) => line.push(TextItem::plain(&t)),
            Event::End(Tag::Item) => {
                if !line.is_empty() {
                    lines.push(line.into())
                }
                if !lines.is_empty() {
                    blocks.push(Block::Paragraph(lines));
                }
                if is_task {
                    return ListItem::Task(blocks, completed, nested_list);
                } else {
                    // check for [, then "\s", then ], then strip front whitespace of other
                    // FIXME: ugly code, probably a nicer and cleaner way to do it

                    if let Some(Block::Paragraph(text)) = blocks.first_mut() {
                        if let Some(Text { inner }) = text.first_mut() {
                            let mut text_iter = inner.iter_mut();
                            if let Some(TextItem::PlainText(t1)) = text_iter.next() {
                                if let Some(TextItem::PlainText(t2)) = text_iter.next() {
                                    if t1.trim() == "[" {
                                        match t2.trim() {
                                            "]" => {
                                                // empty incomplete task
                                                if let Some(TextItem::PlainText(t3)) =
                                                    text_iter.next()
                                                {
                                                    *t3 = t3.trim_start().to_string();
                                                }
                                                *inner = inner[2..].to_vec();
                                                return ListItem::Task(blocks, false, nested_list);
                                            }
                                            "x" | "X" | "" => {
                                                if let Some(TextItem::PlainText(t3)) =
                                                    text_iter.next()
                                                {
                                                    if t3.trim() == "]" {
                                                        let complete = t2.trim() != "";
                                                        if let Some(TextItem::PlainText(t4)) =
                                                            text_iter.next()
                                                        {
                                                            *t4 = t4.trim_start().to_string();
                                                        }
                                                        *inner = inner[3..].to_vec();
                                                        return ListItem::Task(
                                                            blocks,
                                                            complete,
                                                            nested_list,
                                                        );
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return ListItem::Text(blocks, nested_list);
                }
            }
            Event::Start(Tag::List(first_index)) => match first_index {
                None => nested_list = Some(List::Plain(read_list(&mut events_iter))),
                Some(index) => {
                    nested_list = Some(List::Numbered(
                        read_list(&mut events_iter),
                        index.try_into().unwrap(),
                    ))
                }
            },
            Event::TaskListMarker(ticked) => {
                is_task = true;
                completed = ticked;
            }
            Event::Start(Tag::Emphasis) => line.push(read_text_item(events_iter)),
            Event::Start(Tag::Strong) => line.push(read_text_item(events_iter)),
            Event::Start(Tag::Strikethrough) => line.push(read_text_item(events_iter)),
            Event::Code(string) => line.push(TextItem::code(&string)),
            Event::Start(Tag::Link(_inline, url, _title)) => {
                line.push(read_link(events_iter, &url))
            }
            Event::SoftBreak => {
                if !line.is_empty() {
                    lines.push(line.into());
                    line = Vec::new();
                }
            }
            Event::Start(Tag::Paragraph) => {
                blocks.push(Block::Paragraph(read_paragraph(&mut events_iter)))
            }
            Event::Start(Tag::BlockQuote) => {
                blocks.push(Block::BlockQuote(read_blockquote(events_iter)))
            }
            Event::Start(Tag::CodeBlock(lang)) => {
                blocks.push(Block::Code(lang.to_string(), read_code(events_iter)))
            }
            e => {
                error!("read list item reached unimplemented event: {:?}", e);
                unimplemented!()
            }
        }
    }
    ListItem::Text(Vec::new(), None)
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

fn write_paragraph<W: Write>(
    writer: &mut W,
    prefix: &str,
    prefix_first_line: bool,
    lines: &[Text],
) -> Result<(), WriteError> {
    let mut first = true;
    for line in lines {
        let text = format_text(&line);
        if text == "" {
            continue;
        }
        if first && !prefix_first_line {
            first = false;
            writeln!(writer, "{}", format_text(&line))?
        } else {
            writeln!(writer, "{}{}", prefix, format_text(&line))?
        }
    }
    Ok(())
}

fn write_blockquote<W: Write>(
    writer: &mut W,
    prefix: &str,
    blocks: &[Block],
) -> Result<(), WriteError> {
    let prefix = &format!("{}> ", prefix);
    write!(writer, "{}", prefix)?;
    for (i, block) in blocks.iter().enumerate() {
        write_block(writer, prefix, i != 0, block)?;
        if i != blocks.len() - 1 {
            writeln!(writer, "{}", prefix.trim())?;
        }
    }
    Ok(())
}

fn write_code<W: Write>(
    writer: &mut W,
    prefix: &str,
    prefix_first_line: bool,
    lang: &str,
    lines: &[String],
) -> Result<(), WriteError> {
    if prefix_first_line {
        writeln!(writer, "{}```{}", prefix, lang)?;
    } else {
        writeln!(writer, "```{}", lang)?;
    }
    for line in lines {
        writeln!(writer, "{}{}", prefix, line.trim())?
    }
    writeln!(writer, "{}```", prefix)?;
    Ok(())
}

fn write_heading<W: Write>(
    writer: &mut W,
    prefix: &str,
    prefix_first_line: bool,
    text: &Text,
    level: u32,
) -> Result<(), WriteError> {
    if prefix_first_line {
        writeln!(
            writer,
            "{}{}",
            prefix,
            &format!("{} {}", "#".repeat(level as usize), format_text(text))
        )?;
    } else {
        writeln!(
            writer,
            "{}",
            &format!("{} {}", "#".repeat(level as usize), format_text(text))
        )?;
    }
    Ok(())
}

fn format_text(text: &Text) -> String {
    let mut s = String::new();
    for item in text.inner.iter() {
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

fn write_list<W: Write>(writer: &mut W, prefix: &str, list: &List) -> Result<(), WriteError> {
    let child_prefix = &format!("{}    ", prefix);
    let mut index = None;
    let list_items = match list {
        List::Plain(list_items) => list_items,
        List::Numbered(list_items, first_index) => {
            index = Some(*first_index);
            list_items
        }
    };
    for item in list_items {
        if let Some(i) = index {
            write!(writer, "{}{}. ", prefix, i)?;
            index = Some(i + 1)
        } else {
            write!(writer, "{}- ", prefix)?
        }
        match item {
            ListItem::Text(blocks, nested_list) => {
                for (i, block) in blocks.iter().enumerate() {
                    write_block(writer, child_prefix, i != 0, block)?;
                    if i != blocks.len() - 1 {
                        writeln!(writer)?;
                    }
                }
                match nested_list {
                    Some(nl) => write_list(writer, child_prefix, nl)?,
                    None => (),
                }
            }
            ListItem::Task(blocks, completed, nested_list) => {
                if *completed {
                    write!(writer, "[x] ")?
                } else {
                    write!(writer, "[ ] ")?
                }
                for (i, block) in blocks.iter().enumerate() {
                    write_block(writer, prefix, i != 0, block)?;
                    if i != blocks.len() - 1 {
                        writeln!(writer)?;
                    }
                }
                match nested_list {
                    Some(nl) => write_list(writer, child_prefix, nl)?,
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
    /// Wrapper around string slice that makes debug output `{:?}` to print string same way as `{}`.
    /// Used in different `assert*!` macros in combination with `pretty_assertions` crate to make
    /// test failures to show nice diffs.
    #[derive(PartialEq, Eq)]
    #[doc(hidden)]
    pub struct PrettyString<'a>(pub &'a str);

    /// Make diff to display string as multi-line string
    impl<'a> std::fmt::Debug for PrettyString<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str(self.0)
        }
    }

    macro_rules! assert_eq_str {
        ($left:expr, $right:expr) => {
            pretty_assertions::assert_eq!(PrettyString($left), PrettyString($right));
        };
    }

    fn get_test_nodo() -> Nodo {
        let mut builder = NodoBuilder::default();
        builder
            .tags(vec![
                "nodo".to_string(),
                "more tags".to_string(),
                "hey another tag".to_string(),
            ])
            .start_date(NaiveDate::from_ymd(2015, 3, 14))
            .due_date(NaiveDate::from_ymd(2015, 4, 16))
            .title(vec![TextItem::plain("nodo header level 1, is the title")].into())
            .block(Block::List(List::Numbered(
                vec![
                    ListItem::Text(
                        vec![Block::Paragraph(vec![
                            vec![TextItem::plain("list item 1")].into()
                        ])],
                        None,
                    ),
                    ListItem::Text(
                        vec![Block::Paragraph(vec![
                            vec![TextItem::plain("list item 2")].into()
                        ])],
                        None,
                    ),
                    ListItem::Task(
                        vec![Block::Paragraph(vec![vec![TextItem::plain("task")].into()])],
                        false,
                        None,
                    ),
                    ListItem::Task(
                        vec![Block::Paragraph(vec![vec![TextItem::plain(
                            "completed task",
                        )]
                        .into()])],
                        true,
                        None,
                    ),
                ],
                1,
            )))
            .block(Block::Heading(
                vec![TextItem::plain("nodo header with level 2")].into(),
                2,
            ))
            .block(Block::List(List::Plain(vec![
                ListItem::Task(
                    vec![Block::Paragraph(vec![vec![TextItem::plain(
                        "An item to complete",
                    )]
                    .into()])],
                    false,
                    None,
                ),
                ListItem::Task(
                    vec![Block::Paragraph(vec![vec![
                        TextItem::plain("A "),
                        TextItem::emphasis("completed"),
                        TextItem::plain(" item, yay"),
                    ]
                    .into()])],
                    true,
                    Some(List::Plain(vec![
                        ListItem::Task(
                            vec![Block::Paragraph(vec![vec![
                                TextItem::plain("Hey a "),
                                TextItem::strong("nested"),
                                TextItem::plain(" task"),
                            ]
                            .into()])],
                            false,
                            None,
                        ),
                        ListItem::Text(
                            vec![Block::Paragraph(vec![vec![
                                TextItem::plain("And a "),
                                TextItem::emphasis("nested"),
                                TextItem::plain(" text"),
                            ]
                            .into()])],
                            None,
                        ),
                    ])),
                ),
                ListItem::Text(
                    vec![Block::Paragraph(vec![vec![TextItem::plain(
                        "a text list item",
                    )]
                    .into()])],
                    Some(List::Plain(vec![
                        ListItem::Text(
                            vec![Block::Paragraph(vec![vec![
                                TextItem::plain("nested "),
                                TextItem::strong("list"),
                                TextItem::plain(" again"),
                            ]
                            .into()])],
                            None,
                        ),
                        ListItem::Task(
                            vec![Block::Paragraph(vec![vec![
                                TextItem::plain("and a "),
                                TextItem::code("task"),
                            ]
                            .into()])],
                            false,
                            None,
                        ),
                    ])),
                ),
                ListItem::Task(
                    vec![Block::Paragraph(vec![vec![TextItem::plain(
                        "or a ~task~ list item",
                    )]
                    .into()])],
                    false,
                    None,
                ),
                ListItem::Task(
                    vec![Block::Paragraph(vec![vec![
                        TextItem::plain("and a "),
                        TextItem::strikethrough("technically"),
                        TextItem::plain(" ill-formed task, but should be allowed really"),
                    ]
                    .into()])],
                    false,
                    None,
                ),
            ])));
        builder.build()
    }

    static TEST_NODO_UNFORMATTED: &str = "---
tags: nodo, more tags, hey another tag
start_date: 14/03/2015
due_date: 16/04/2015
---

# nodo header level 1, is the title

1. list item 1
2. list item 2
3. [ ] task
4. [x] completed task

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
start_date: 14/03/2015
due_date: 16/04/2015
---

# nodo header level 1, is the title

1. list item 1
2. list item 2
3. [ ] task
4. [x] completed task

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
            Markdown
                .read(
                    NodoBuilder::default(),
                    &mut TEST_NODO_UNFORMATTED.as_bytes(),
                    &Config::new()
                )
                .unwrap(),
            Markdown
                .read(
                    NodoBuilder::default(),
                    &mut TEST_NODO_FORMATTED.as_bytes(),
                    &Config::new()
                )
                .unwrap()
        )
    }

    #[test]
    fn test_read() {
        assert_eq!(
            Markdown
                .read(
                    NodoBuilder::default(),
                    &mut TEST_NODO_FORMATTED.as_bytes(),
                    &Config::new()
                )
                .unwrap(),
            get_test_nodo(),
        );
        assert_eq!(
            Markdown
                .read(
                    NodoBuilder::default(),
                    &mut TEST_NODO_UNFORMATTED.as_bytes(),
                    &Config::new()
                )
                .unwrap(),
            get_test_nodo(),
        );
    }

    #[test]
    fn test_write() {
        let mut writer: Vec<u8> = Vec::new();
        Markdown
            .write(&get_test_nodo(), &mut writer, &Config::new())
            .unwrap();
        assert_eq_str!(&String::from_utf8(writer).unwrap(), TEST_NODO_FORMATTED);
    }

    #[test]
    fn test_write_read_gives_same_nodo() {
        let mut s = Vec::new();
        Markdown
            .write(&get_test_nodo(), &mut s, &Config::new())
            .unwrap();
        assert_eq!(
            Markdown
                .read(NodoBuilder::default(), &mut &s[..], &Config::new())
                .unwrap(),
            get_test_nodo()
        );
    }

    #[test]
    fn test_read_write_gives_same_output() {
        let nodo = Markdown
            .read(
                NodoBuilder::default(),
                &mut TEST_NODO_FORMATTED.as_bytes(),
                &Config::new(),
            )
            .unwrap();
        let mut s = Vec::new();

        Markdown.write(&nodo, &mut s, &Config::new()).unwrap();
        assert_eq_str!(&String::from_utf8(s).unwrap(), TEST_NODO_FORMATTED);
    }

    #[test]
    fn test_read_without_frontmatter() {
        let s = "# title";
        let mut builder = NodoBuilder::default();
        builder.title(vec![TextItem::plain("title")].into());
        assert_eq!(
            Markdown
                .read(NodoBuilder::default(), &mut s.as_bytes(), &Config::new())
                .unwrap(),
            builder.build()
        )
    }

    const LARGE_MD_STRING:&str = r#"---
tags: nodo, more tags, hey another tag
start_date: 14/03/2015
due_date: 16/04/2015
---

# Markdown: Syntax

- [Overview](#overview)
    - [Philosophy](#philosophy)
    - [Inline HTML](#html)
    - [Automatic Escaping for Special Characters](#autoescape)
- [Block Elements](#block)
    - [Paragraphs and Line Breaks](#p)
    - [Headers](#header)
    - [Blockquotes](#blockquote)
    - [Lists](#list)
    - [Code Blocks](#precode)
    - [Horizontal Rules](#hr)
- [Span Elements](#span)
    - [Links](#link)
    - [Emphasis](#em)
    - [Code](#code)
    - [Images](#img)
- [Miscellaneous](#misc)
    - [Backslash Escapes](#backslash)
    - [Automatic Links](#autolink)

**Note:** This document is itself written using Markdown; you
can [see the source for it by adding '.text' to the URL](/projects/markdown/syntax.text).

---

## Overview

### Philosophy

Markdown is intended to be as easy-to-read and easy-to-write as is feasible.

Readability, however, is emphasized above all else. A Markdown-formatted
document should be publishable as-is, as plain text, without looking
like it's been marked up with tags or formatting instructions. While
Markdown's syntax has been influenced by several existing text-to-HTML
filters -- including [Setext](http://docutils.sourceforge.net/mirror/setext.html), [atx](http://www.aaronsw.com/2002/atx/), [Textile](http://textism.com/tools/textile/), [reStructuredText](http://docutils.sourceforge.net/rst.html),
[Grutatext](http://www.triptico.com/software/grutatxt.html), and [EtText](http://ettext.taint.org/doc/) -- the single biggest source of
inspiration for Markdown's syntax is the format of plain text email.

## Block Elements

### Paragraphs and Line Breaks

A paragraph is simply one or more consecutive lines of text, separated
by one or more blank lines. (A blank line is any line that looks like a
blank line -- a line containing nothing but spaces or tabs is considered
blank.) Normal paragraphs should not be indented with spaces or tabs.

The implication of the "one or more consecutive lines of text" rule is
that Markdown supports "hard-wrapped" text paragraphs. This differs
significantly from most other text-to-HTML formatters (including Movable
Type's "Convert Line Breaks" option) which translate every line break
character in a paragraph into a `<br />` tag.

When you *do* want to insert a `<br />` break tag using Markdown, you
end a line with two or more spaces, then type return.

### Headers

Markdown supports two styles of headers, [Setext] [1] and [atx] [2].

Optionally, you may "close" atx-style headers. This is purely
cosmetic -- you can use this if you think it looks better. The
closing hashes don't even need to match the number of hashes
used to open the header. (The number of opening hashes
determines the header level.)

### Blockquotes

Markdown uses email-style `>` characters for blockquoting. If you're
familiar with quoting passages of text in an email message, then you
know how to create a blockquote in Markdown. It looks best if you hard
wrap the text and put a `>` before every line:

> This is a blockquote with two paragraphs. Lorem ipsum dolor sit amet,
> consectetuer adipiscing elit. Aliquam hendrerit mi posuere lectus.
> Vestibulum enim wisi, viverra nec, fringilla in, laoreet vitae, risus.
>
> Donec sit amet nisl. Aliquam semper ipsum sit amet velit. Suspendisse
> id sem consectetuer libero luctus adipiscing.

Markdown allows you to be lazy and only put the `>` before the first
line of a hard-wrapped paragraph:

> This is a blockquote with two paragraphs. Lorem ipsum dolor sit amet,
> consectetuer adipiscing elit. Aliquam hendrerit mi posuere lectus.
> Vestibulum enim wisi, viverra nec, fringilla in, laoreet vitae, risus.

> Donec sit amet nisl. Aliquam semper ipsum sit amet velit. Suspendisse
> id sem consectetuer libero luctus adipiscing.

Blockquotes can be nested (i.e. a blockquote-in-a-blockquote) by
adding additional levels of `>`:

> This is the first level of quoting.
>
> > This is nested blockquote.
>
> Back to the first level.

Blockquotes can contain other Markdown elements, including headers, lists,
and code blocks:

> ## This is a header.
>
> 1. This is the first list item.
> 2. This is the second list item.
>
> Here's some example code:
>
> ```
> return shell_exec("echo $input | $markdown_script");
> ```

Any decent text editor should make email-style quoting easy. For
example, with BBEdit, you can make a selection and choose Increase
Quote Level from the Text menu.

### Lists

Markdown supports ordered (numbered) and unordered (bulleted) lists.

Unordered lists use asterisks, pluses, and hyphens -- interchangably
-- as list markers:

- Red
- Green
- Blue

is equivalent to:

- Red
- Green
- Blue

and:

- Red
- Green
- Blue

Ordered lists use numbers followed by periods:

1. Bird
2. McHale
3. Parish

It's important to note that the actual numbers you use to mark the
list have no effect on the HTML output Markdown produces. The HTML
Markdown produces from the above list is:

If you instead wrote the list in Markdown like this:

1. Bird
2. McHale
3. Parish

or even:

3. Bird
4. McHale
5. Parish

you'd get the exact same HTML output. The point is, if you want to,
you can use ordinal numbers in your ordered Markdown lists, so that
the numbers in your source match the numbers in your published HTML.
But if you want to be lazy, you don't have to.

To make lists look nice, you can wrap items with hanging indents:

- Lorem ipsum dolor sit amet, consectetuer adipiscing elit.
    Aliquam hendrerit mi posuere lectus. Vestibulum enim wisi,
    viverra nec, fringilla in, laoreet vitae, risus.
- Donec sit amet nisl. Aliquam semper ipsum sit amet velit.
    Suspendisse id sem consectetuer libero luctus adipiscing.

But if you want to be lazy, you don't have to:

- Lorem ipsum dolor sit amet, consectetuer adipiscing elit.
    Aliquam hendrerit mi posuere lectus. Vestibulum enim wisi,
    viverra nec, fringilla in, laoreet vitae, risus.
- Donec sit amet nisl. Aliquam semper ipsum sit amet velit.
    Suspendisse id sem consectetuer libero luctus adipiscing.

List items may consist of multiple paragraphs. Each subsequent
paragraph in a list item must be indented by either 4 spaces
or one tab:

1. This is a list item with two paragraphs. Lorem ipsum dolor
    sit amet, consectetuer adipiscing elit. Aliquam hendrerit
    mi posuere lectus.

    Vestibulum enim wisi, viverra nec, fringilla in, laoreet
    vitae, risus. Donec sit amet nisl. Aliquam semper ipsum
    sit amet velit.
2. Suspendisse id sem consectetuer libero luctus adipiscing.

It looks nice if you indent every line of the subsequent
paragraphs, but here again, Markdown will allow you to be
lazy:

- This is a list item with two paragraphs.

    This is the second paragraph in the list item. You're
    only required to indent the first line. Lorem ipsum dolor
    sit amet, consectetuer adipiscing elit.
- Another item in the same list.

To put a blockquote within a list item, the blockquote's `>`
delimiters need to be indented:

- A list item with a blockquote:

    > This is a blockquote
    > inside a list item.

To put a code block within a list item, the code block needs
to be indented *twice* -- 8 spaces or two tabs:

- A list item with a code block:

    ```
    <code goes here>
    ```

### Code Blocks

Pre-formatted code blocks are used for writing about programming or
markup source code. Rather than forming normal paragraphs, the lines
of a code block are interpreted literally. Markdown wraps a code block
in both `<pre>` and `<code>` tags.

To produce a code block in Markdown, simply indent every line of the
block by at least 4 spaces or 1 tab.

This is a normal paragraph:

```
This is a code block.
```

Here is an example of AppleScript:

```
tell application "Foo"
    beep
end tell
```

A code block continues until it reaches a line that is not indented
(or the end of the article).

Within a code block, ampersands (`&`) and angle brackets (`<` and `>`)
are automatically converted into HTML entities. This makes it very
easy to include example HTML source code using Markdown -- just paste
it and indent it, and Markdown will handle the hassle of encoding the
ampersands and angle brackets. For example, this:

```
<div class="footer">
    &copy; 2004 Foo Corporation
</div>
```

Regular Markdown syntax is not processed within code blocks. E.g.,
asterisks are just literal asterisks within a code block. This means
it's also easy to use Markdown to write about Markdown's own syntax.

```
tell application "Foo"
    beep
end tell
```

## Span Elements

### Links

Markdown supports two style of links: *inline* and *reference*.

In both styles, the link text is delimited by [square brackets].

To create an inline link, use a set of regular parentheses immediately
after the link text's closing square bracket. Inside the parentheses,
put the URL where you want the link to point, along with an *optional*
title for the link, surrounded in quotes. For example:

This is [an example](http://example.com/) inline link.

[This link](http://example.net/) has no title attribute.

### Emphasis

Markdown treats asterisks (`*`) and underscores (`_`) as indicators of
emphasis. Text wrapped with one `*` or `_` will be wrapped with an
HTML `<em>` tag; double `*`'s or `_`'s will be wrapped with an HTML
`<strong>` tag. E.g., this input:

*single asterisks*

*single underscores*

**double asterisks**

**double underscores**

### Code

To indicate a span of code, wrap it with backtick quotes (`c`).
Unlike a pre-formatted code block, a code span indicates code within a
normal paragraph. For example:

Use the `printf()` function.
"#;
    #[test]
    fn test_commonmark_parses() {
        let nodo = Markdown
            .read(
                NodoBuilder::default(),
                &mut LARGE_MD_STRING.as_bytes(),
                &Config::new(),
            )
            .unwrap();
        let mut s = Vec::new();

        Markdown.write(&nodo, &mut s, &Config::new()).unwrap();
        let comp_string = String::from_utf8(s).unwrap();
        assert_eq_str!(&comp_string, LARGE_MD_STRING);
    }

    #[test]
    fn test_write_doesnt_change_nodo() {
        let nodo1 = Markdown
            .read(
                NodoBuilder::default(),
                &mut LARGE_MD_STRING.as_bytes(),
                &Config::new(),
            )
            .unwrap();
        let mut s = Vec::new();
        Markdown.write(&nodo1, &mut s, &Config::new()).unwrap();

        let nodo2 = Markdown
            .read(NodoBuilder::default(), &mut &s[..], &Config::new())
            .unwrap();
        assert_eq!(nodo1, nodo2)
    }
}
