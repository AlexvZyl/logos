use crate::{
    config::get_translations,
    filesystem::{decompress_xz, is_xml_file, is_xz_compressed_xml},
    prelude::*,
};
use indexmap::IndexMap;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::time::Instant;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Bible {
    translation: String,
    disk_file: PathBuf,

    metadata: Metadata,
    index: IndexMap<String, Book>,
    books: Vec<String>,
    raw: String,
}

impl Bible {
    pub fn get_book(&self, book_name: &str) -> Book {
        // TODO: Don't like the clone and unwrap.
        self.index.get(book_name).unwrap().clone()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Work level metadata from the OSIS header.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Metadata {
    pub work: String,
    pub title: String,
    pub description: String,
    pub identifier: String,
    /// `xml:lang` of the text.
    pub language: String,
    pub languages: Vec<Typed>,
    pub publishers: Vec<Typed>,
    pub rights: Vec<Typed>,
    pub ref_system: String,
    pub revisions: Vec<Revision>,
}

/// A header value with its OSIS `type` attribute, e.g. `x-copyright`.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Typed {
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Revision {
    pub date: String,
    pub description: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default)]
pub struct Book {
    pub chapters: Vec<Chapter>,
}

impl Book {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_chapters(&self) -> impl Iterator<Item = &Chapter> {
        self.chapters.iter()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Chapter {
    pub number: usize,
    pub verses: Vec<VerseView>,
}

impl Chapter {
    pub fn new(number: usize) -> Self {
        Self {
            number,
            verses: Vec::new(),
        }
    }

    pub fn get_verses(&self) -> impl Iterator<Item = &VerseView> {
        self.verses.iter()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// A non-owning view into the raw memory.
#[derive(Debug, Clone)]
pub struct VerseView {
    pub number: usize,
    /// (start, end) byte ranges of text regions in the raw data.
    pub indices: Vec<(usize, usize)>,
}

impl VerseView {
    pub fn new(number: usize) -> Self {
        Self {
            number,
            indices: Vec::new(),
        }
    }

    // TODO: Don't really like that you have to pass in the raw.
    // Collect all of these types (Bible, chapter, verse) into a single
    // struct with references.
    pub fn collect_string(&self, raw: &str) -> String {
        // There seems to be some groups of whitespaces in the data.  I guess the file is just
        // dirty.
        // TODO: Investigate and move this logic out.
        let s: String = self.indices.iter().map(move |&(s, e)| &raw[s..e]).collect();
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Bible {
    pub fn from_translation(translation: &str) -> Result<Bible> {
        let file = get_translations()
            // TODO: Select translation from UI.
            .get("KJV")
            .ok_or(Error::UnsupprtedTranslation(translation.to_string()))?;
        Bible::from_file(file)
    }

    pub fn from_file(path: &std::path::Path) -> Result<Bible> {
        info!("Loading {:?} into memory", path);
        let start = Instant::now();
        let raw = if is_xz_compressed_xml(path) {
            decompress_xz(path)?
        } else if is_xml_file(path) {
            std::fs::read_to_string(path)?
        } else {
            return Err(Error::InvalidBibleFile);
        };
        info!("Loaded {:?} in {:?}", path, start.elapsed());

        let metadata = Self::parse_metadata(&raw)?;
        let index = Self::build_index_from_osis(&raw)?;
        return Ok(Bible {
            disk_file: path.to_path_buf(),
            metadata,
            books: index.keys().cloned().collect(),
            index: index,
            translation: "KJV".to_string(), // TODO: Get translation.
            raw: raw,
        });
    }

    pub fn get_books(&self) -> &Vec<String> {
        &self.books
    }

    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn get_raw_data(&self) -> &str {
        &self.raw
    }

    pub fn get_book_index(&self, name: &str) -> Result<&Book> {
        self.index
            .get(name)
            .ok_or(Error::BookNotFound(name.to_string()))
    }

    fn build_index_from_osis(raw: &str) -> Result<IndexMap<String, Book>> {
        info!("Building bible index");

        let start = Instant::now();
        let mut index: IndexMap<String, Book> = IndexMap::new();
        let mut reader = Reader::from_str(raw);

        let mut buf = Vec::new();
        let mut book = String::new();
        let mut awaiting_title = false;
        let mut in_verse = false;
        let mut current_chapter = 1;
        let mut current_verse = 1;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e))
                    if e.name().as_ref() == b"div" && Self::has_attr_val(e, b"type", b"book") =>
                {
                    awaiting_title = true;
                }
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"title" && awaiting_title => {
                    if let Some(name) = Self::attr(e, b"short") {
                        book = name;
                        index.entry(book.clone()).or_default();
                    }
                    awaiting_title = false;
                    current_chapter = 1;
                }
                Ok(Event::Empty(ref e))
                    if e.name().as_ref() == b"chapter" && Self::has_attr(e, b"sID") =>
                {
                    index
                        .entry(book.clone())
                        .or_default()
                        .chapters
                        .push(Chapter::new(current_chapter));
                    current_chapter += 1;
                    current_verse = 1;
                }
                Ok(Event::Empty(ref e))
                    if e.name().as_ref() == b"verse" && Self::has_attr(e, b"sID") =>
                {
                    in_verse = true;
                    if let Some(ch) = index.get_mut(&book).and_then(|b| b.chapters.last_mut()) {
                        ch.verses.push(VerseView::new(current_verse));
                        current_verse += 1;
                    }
                }
                Ok(Event::Empty(ref e))
                    if e.name().as_ref() == b"verse" && Self::has_attr(e, b"eID") =>
                {
                    in_verse = false;
                }
                Ok(Event::Text(ref e)) if in_verse => {
                    let len = e.len();
                    if len > 0 {
                        let end = reader.buffer_position() as usize;
                        let start = end - len;
                        if let Some(verse) = index
                            .get_mut(&book)
                            .and_then(|b| b.chapters.last_mut())
                            .and_then(|c| c.verses.last_mut())
                        {
                            verse.indices.push((start, end));
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::BibleIndex(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        info!("Built index in {:?}", start.elapsed());
        Ok(index)
    }

    /// Parses the work level metadata from the OSIS header.
    fn parse_metadata(raw: &str) -> Result<Metadata> {
        let mut metadata = Metadata::default();
        let mut reader = Reader::from_str(raw);

        let mut buf = Vec::new();
        let mut in_header = false;
        let mut in_work = false;
        // Element (and its `type` attribute) whose text is being collected.
        let mut capture: Option<(Vec<u8>, String, String)> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(ref e)) if capture.is_some() => {
                    if let Some((_, _, text)) = &mut capture {
                        text.push_str(&String::from_utf8_lossy(e));
                    }
                }
                Ok(Event::End(ref e))
                    if capture.as_ref().is_some_and(|(el, _, _)| el == e.name().as_ref()) =>
                {
                    let (element, kind, text) = capture.take().expect("Checked above");
                    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
                    Self::set_header_field(&mut metadata, in_work, &element, kind, text);
                }
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"osisText" => {
                    metadata.language = Self::attr(e, b"xml:lang").unwrap_or_default();
                }
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"header" => in_header = true,
                // Everything we need is in the header.
                Ok(Event::End(ref e)) if e.name().as_ref() == b"header" => break,
                Ok(Event::End(ref e)) if e.name().as_ref() == b"work" => in_work = false,
                Ok(Event::Start(ref e)) if in_header => match e.name().as_ref() {
                    b"work" if metadata.work.is_empty() => {
                        in_work = true;
                        metadata.work = Self::attr(e, b"osisWork").unwrap_or_default();
                    }
                    b"revisionDesc" => metadata.revisions.push(Revision::default()),
                    name => {
                        let kind = Self::attr(e, b"type").unwrap_or_default();
                        capture = Some((name.to_vec(), kind, String::new()));
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::BibleIndex(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(metadata)
    }

    fn set_header_field(
        metadata: &mut Metadata,
        in_work: bool,
        element: &[u8],
        kind: String,
        text: String,
    ) {
        match element {
            b"date" => {
                if let Some(r) = metadata.revisions.last_mut() {
                    r.date = text;
                }
            }
            b"p" => {
                if let Some(r) = metadata.revisions.last_mut() {
                    r.description = text;
                }
            }
            _ if !in_work => {}
            b"title" => metadata.title = text,
            b"description" => metadata.description = text,
            b"identifier" => metadata.identifier = text,
            b"refSystem" => metadata.ref_system = text,
            b"language" => metadata.languages.push(Typed { kind, text }),
            b"publisher" => metadata.publishers.push(Typed { kind, text }),
            b"rights" => metadata.rights.push(Typed { kind, text }),
            _ => {}
        }
    }

    // Parsing utils.

    fn attr(e: &BytesStart, key: &[u8]) -> Option<String> {
        e.attributes()
            .filter_map(|a| a.ok())
            .find(|a| a.key.as_ref() == key)
            .map(|a| String::from_utf8_lossy(&a.value).to_string())
    }

    fn has_attr(e: &BytesStart, key: &[u8]) -> bool {
        e.attributes()
            .filter_map(|a| a.ok())
            .any(|a| a.key.as_ref() == key)
    }

    fn has_attr_val(e: &BytesStart, key: &[u8], val: &[u8]) -> bool {
        e.attributes()
            .filter_map(|a| a.ok())
            .any(|a| a.key.as_ref() == key && a.value.as_ref() == val)
    }
}

