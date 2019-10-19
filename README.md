# Nodo

Notes and todos, together.

## About

Nodo is a program that aims to make notes and todos for developers and every
day life somewhat easier. Notes often contain context about something and todos
are often very short. However, without management of what the note refers to it
has little use and can be easily forgotten. Todos also typically need context
which may be forgotten if not carefully noted down. They therefore are
intertwined - producing Nodo.

## Principles

Besides aiming to bring notes and todos together there are some more technical
aims:

- Provide a comfortable cli interface for quick interactions while still
  facilitating longer, more in-depth operations

- Allow multiple different file formats to be used, via swappable backends

- Have a central storage location for nodos but also allow keeping them local
  to a project

- Keep it all in plaintext, this keeps things simple and so easy to share via
  messaging, email or even adding to source control

- Be fast and efficient, somewhat secondary to a clean and efficient UI and UX
  is to have the actual program be fast and efficient, hence Rust.

### Layout

Projects can be nested as directories on the filesystem, each actual file
should have an extension on the filesystem but this extension can be omitted
during commands.

## Why another app?

Lots of apps tend to have a large focus on todos and so have lots of emphasis
on short 'reminders'.

Keeping things together and in plaintext files brings lots of advantages:

- When in an editing session, just keep the file open to manage the todo!

- Easily shareable

- Adding detail to a nodo is simple and quick, just another line of text in a
  file

## Why Rust?

Correctness, Efficiency, Performance.
