use crate::{
    config::get_translations,
    filesystem::{decompress_xz, is_xml_file, is_xz_compressed_xml},
    prelude::*,
};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::time::Instant;

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Owns the raw file.  Everything else is a `RawSpan` into it.
#[derive(Debug)]
pub struct Bible {
    translation: String,
    disk_file: PathBuf,

    metadata: Metadata,
    /// In file order.
    books: Vec<Book>,
    raw: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// A byte range into `Bible::raw`.  Empty means not present in the file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RawSpan {
    start: u32,
    end: u32,
}

impl RawSpan {
    fn new(start: usize, end: usize) -> Self {
        assert!(start <= end);
        assert!(end <= u32::MAX as usize);
        Self {
            start: start as u32,
            end: end as u32,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Work level metadata from the OSIS header.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Metadata {
    pub work: RawSpan,
    pub title: RawSpan,
    pub description: RawSpan,
    pub identifier: RawSpan,
    /// `xml:lang` of the text.
    pub language: RawSpan,
    pub ref_system: RawSpan,
    pub languages: Vec<Typed>,
    pub publishers: Vec<Typed>,
    pub rights: Vec<Typed>,
    pub revisions: Vec<Revision>,
}

/// A header value with its OSIS `type` attribute, e.g. `x-copyright`.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Typed {
    pub kind: RawSpan,
    pub text: RawSpan,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Revision {
    pub date: RawSpan,
    pub description: RawSpan,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct Book {
    /// Short name, e.g. `Genesis`.
    pub name: RawSpan,
    pub chapters: Vec<Chapter>,
}

impl Book {
    pub fn get_chapters(&self) -> impl Iterator<Item = &Chapter> {
        self.chapters.iter()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Chapter {
    pub number: usize,
    pub verses: Vec<Verse>,
}

impl Chapter {
    pub fn new(number: usize) -> Self {
        Self {
            number,
            verses: Vec::new(),
        }
    }

    pub fn get_verses(&self) -> impl Iterator<Item = &Verse> {
        self.verses.iter()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Verse {
    pub number: usize,
    /// Inline markup splits the text into multiple spans.
    pub text: Vec<RawSpan>,
}

impl Verse {
    pub fn new(number: usize) -> Self {
        Self {
            number,
            text: Vec::new(),
        }
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

        if raw.len() > u32::MAX as usize {
            return Err(Error::InvalidBibleFile);
        }
        let metadata = Self::parse_metadata(&raw)?;
        let books = Self::build_index_from_osis(&raw)?;
        return Ok(Bible {
            disk_file: path.to_path_buf(),
            metadata,
            books,
            translation: "KJV".to_string(), // TODO: Get translation.
            raw: raw,
        });
    }

    // Accessors.

    /// The exact bytes from the file.
    pub fn raw(&self, span: RawSpan) -> &str {
        &self.raw[span.start as usize..span.end as usize]
    }

    /// Display text: entities unescaped and whitespace collapsed.
    pub fn text(&self, spans: &[RawSpan]) -> String {
        // There seems to be some groups of whitespaces in the data.  I guess the file is just
        // dirty.
        let s: String = spans.iter().map(|&s| self.raw(s)).collect();
        let s = quick_xml::escape::unescape(&s).unwrap_or(s.as_str().into());
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[allow(dead_code)]
    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn book_names(&self) -> Vec<String> {
        self.books.iter().map(|b| self.raw(b.name).to_string()).collect()
    }

    pub fn get_book(&self, name: &str) -> Result<&Book> {
        self.books
            .iter()
            .find(|b| self.raw(b.name) == name)
            .ok_or(Error::BookNotFound(name.to_string()))
    }

    // Parsing.

    /// Parses the work level metadata from the OSIS header.
    fn parse_metadata(raw: &str) -> Result<Metadata> {
        let mut metadata = Metadata::default();
        let mut reader = Reader::from_str(raw);

        let mut in_header = false;
        let mut in_work = false;
        let mut works = 0;
        // Element, its `type` attribute and the start of its content.
        let mut capture: Option<(&[u8], RawSpan, usize)> = None;

        loop {
            // Start of the markup of the event about to be read.
            let before = reader.buffer_position() as usize;
            let event = reader
                .read_event()
                .map_err(|e| Error::BibleIndex(e.to_string()))?;
            let after = reader.buffer_position() as usize;

            match event {
                Event::Start(ref e) if e.name().as_ref() == b"osisText" => {
                    metadata.language = Self::attr_span(raw, e, b"xml:lang").unwrap_or_default();
                }
                Event::Start(ref e) if e.name().as_ref() == b"header" => in_header = true,
                // Everything we need is in the header.
                Event::End(ref e) if e.name().as_ref() == b"header" => break,
                Event::Start(ref e) if in_header && e.name().as_ref() == b"work" => {
                    // Only the first work describes this text.
                    works += 1;
                    in_work = works == 1;
                    if in_work {
                        metadata.work = Self::attr_span(raw, e, b"osisWork").unwrap_or_default();
                    }
                }
                Event::End(ref e) if e.name().as_ref() == b"work" => in_work = false,
                Event::Start(ref e) if in_header && e.name().as_ref() == b"revisionDesc" => {
                    metadata.revisions.push(Revision::default())
                }
                Event::Start(ref e) if in_header && capture.is_none() => {
                    let kind = Self::attr_span(raw, e, b"type").unwrap_or_default();
                    let name = &raw.as_bytes()[before + 1..before + 1 + e.name().as_ref().len()];
                    capture = Some((name, kind, after));
                }
                Event::End(ref e)
                    if capture.is_some_and(|(name, _, _)| name == e.name().as_ref()) =>
                {
                    let (name, kind, start) = capture.take().expect("Checked above");
                    let text = RawSpan::new(start, before);
                    Self::set_header_field(&mut metadata, in_work, name, kind, text);
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(metadata)
    }

    fn set_header_field(
        metadata: &mut Metadata,
        in_work: bool,
        element: &[u8],
        kind: RawSpan,
        text: RawSpan,
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

    fn build_index_from_osis(raw: &str) -> Result<Vec<Book>> {
        info!("Building bible index");

        let start = Instant::now();
        let mut books: Vec<Book> = Vec::new();
        let mut reader = Reader::from_str(raw);

        let mut awaiting_title = false;
        let mut in_verse = false;
        let mut current_chapter = 1;
        let mut current_verse = 1;

        loop {
            let before = reader.buffer_position() as usize;
            let event = reader
                .read_event()
                .map_err(|e| Error::BibleIndex(e.to_string()))?;
            let after = reader.buffer_position() as usize;

            match event {
                Event::Start(ref e)
                    if e.name().as_ref() == b"div" && Self::has_attr_val(e, b"type", b"book") =>
                {
                    awaiting_title = true;
                }
                Event::Start(ref e) if e.name().as_ref() == b"title" && awaiting_title => {
                    if let Some(name) = Self::attr_span(raw, e, b"short") {
                        books.push(Book {
                            name,
                            chapters: Vec::new(),
                        });
                    }
                    awaiting_title = false;
                    current_chapter = 1;
                }
                Event::Empty(ref e)
                    if e.name().as_ref() == b"chapter" && Self::has_attr(e, b"sID") =>
                {
                    if let Some(book) = books.last_mut() {
                        book.chapters.push(Chapter::new(current_chapter));
                    }
                    current_chapter += 1;
                    current_verse = 1;
                }
                Event::Empty(ref e)
                    if e.name().as_ref() == b"verse" && Self::has_attr(e, b"sID") =>
                {
                    assert!(!in_verse);
                    in_verse = true;
                    if let Some(ch) = books.last_mut().and_then(|b| b.chapters.last_mut()) {
                        ch.verses.push(Verse::new(current_verse));
                        current_verse += 1;
                    }
                }
                Event::Empty(ref e)
                    if e.name().as_ref() == b"verse" && Self::has_attr(e, b"eID") =>
                {
                    assert!(in_verse);
                    in_verse = false;
                }
                // Entities arrive as their own event, keep them escaped in the span.
                Event::Text(_) | Event::GeneralRef(_) if in_verse && after > before => {
                    if let Some(verse) = books
                        .last_mut()
                        .and_then(|b| b.chapters.last_mut())
                        .and_then(|c| c.verses.last_mut())
                    {
                        verse.text.push(RawSpan::new(before, after));
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        assert!(!in_verse);
        info!("Built index in {:?}", start.elapsed());
        Ok(books)
    }

    // Parsing utils.

    /// Span of an attribute value.  Only valid for events borrowed from `raw`.
    fn attr_span(raw: &str, e: &BytesStart, key: &[u8]) -> Option<RawSpan> {
        let a = e
            .attributes()
            .filter_map(|a| a.ok())
            .find(|a| a.key.as_ref() == key)?;
        // An owned value is a copy, so its pointer says nothing about where it is in `raw`.
        if !matches!(a.value, std::borrow::Cow::Borrowed(_)) {
            return None;
        }
        let start = (a.value.as_ptr() as usize).checked_sub(raw.as_ptr() as usize)?;
        let end = start + a.value.len();
        (end <= raw.len()).then(|| RawSpan::new(start, end))
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

////////////////////////////////////////////////////////////////////////////////////////////////////
