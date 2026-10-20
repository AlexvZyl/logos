use crate::app::events::UserAction;
use crate::bible::{Bible, BookId};
use crate::components::book_column::Position;
use crate::prelude::*;

/// Commands from the prompt that are not plain user actions.
pub enum Command {
    /// `:john 3:16`, `:1 sam 2`, `:gen`.
    Goto { book: BookId, position: Position },
}

/// Parses the prompt input, without the leading `:`.  Commands that are user actions, such as
/// `:q`, become that action so the modes handle them as if the key was pressed.  `Ok(None)` for
/// empty input.  The error is a message for the user.
pub fn parse(input: &str, bible: &Bible) -> std::result::Result<Option<AppEvent>, String> {
    let input = input.trim();
    match input {
        "" => Ok(None),
        "q" | "quit" => Ok(Some(AppEvent::UserAction(UserAction::Quit))),
        _ => goto(input, bible).map(|command| Some(AppEvent::Command(command))),
    }
}

/// `<book> [chapter[:verse]]`, where the book may contain spaces and digits, e.g. `1 john 3`.
fn goto(input: &str, bible: &Bible) -> std::result::Result<Command, String> {
    let (book, reference) = match input.rsplit_once(char::is_whitespace) {
        Some((book, last)) if last.starts_with(|c: char| c.is_ascii_digit()) => (book, Some(last)),
        _ => (input, None),
    };
    let id = find_book(book, bible)?;
    let Some(reference) = reference else {
        return Ok(Command::Goto { book: id, position: Position::default() });
    };

    let invalid = || format!("Invalid reference: {reference}");
    let (chapter, verse) = match reference.split_once(':') {
        Some((chapter, verse)) => (chapter, Some(verse.parse().map_err(|_| invalid())?)),
        None => (reference, None),
    };
    let chapter: usize = chapter.parse().map_err(|_| invalid())?;

    let found = bible.get_book(&id).map_err(|e| e.to_string())?;
    let name = bible.raw(found.name);
    let Some(found) = found.get_chapters().find(|c| c.number == chapter) else {
        return Err(format!("{name} has no chapter {chapter}"));
    };
    if let Some(verse) = verse
        && !found.get_verses().any(|v| v.number == verse)
    {
        return Err(format!("{name} {chapter} has no verse {verse}"));
    }

    // Verse `0` is the chapter heading.
    let position = Position { chapter, verse: verse.unwrap_or(0) };
    Ok(Command::Goto { book: id, position })
}

/// Matches the id or name, ignoring case, spaces and punctuation.  An exact match wins over a
/// prefix, so `phil` is Philippians and not Philemon.  A prefix must match a single book.
fn find_book(query: &str, bible: &Bible) -> std::result::Result<BookId, String> {
    let normalize = |s: &str| -> String {
        s.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
    };
    let key = normalize(query);
    if key.is_empty() {
        return Err("Missing book".to_string());
    }

    let books: Vec<(BookId, String, String)> = bible
        .book_groups()
        .into_iter()
        .flat_map(|(_, books)| books)
        .map(|(id, name)| {
            let (id_key, name_key) = (normalize(id.as_str()), normalize(&name));
            (id, id_key, name_key)
        })
        .collect();

    if let Some((id, _, _)) = books.iter().find(|(_, i, n)| *i == key || *n == key) {
        return Ok(id.clone());
    }
    let matches: Vec<&BookId> = books
        .iter()
        .filter(|(_, i, n)| i.starts_with(&key) || n.starts_with(&key))
        .map(|(id, _, _)| id)
        .collect();
    match matches.as_slice() {
        [id] => Ok((*id).clone()),
        [] => Err(format!("No book matches {query}")),
        _ => Err(format!("{query} matches {} books", matches.len())),
    }
}
