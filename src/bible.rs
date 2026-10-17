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

/// Work level metadata from the OSIS header.  Examples are from the KJV.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Metadata {
    /// OSIS name of the work, e.g. `Bible.en.kjv`.
    pub work: RawSpan,
    /// Name of the translation, e.g. `King James Version`.
    pub title: RawSpan,
    /// A sentence or two about the translation.
    pub description: RawSpan,
    /// Short id from the distributor, e.g. `eng-kjv` from eBible.org.
    pub identifier: RawSpan,
    /// `xml:lang` of the text, e.g. `en`.
    pub language: RawSpan,
    /// Versification the references follow, e.g. `Bible`.
    pub ref_system: RawSpan,
    /// The language described in different ways, e.g. `Ethnologue` is `eng`.
    pub languages: Vec<Typed<LanguageKind>>,
    /// Print and electronic publishers, e.g. `Electronic` is `eBible.org`.
    pub publishers: Vec<Typed<PublisherKind>>,
    /// Copyright and license, e.g. `Copyright` is `public domain`.
    pub rights: Vec<Typed<RightsKind>>,
    /// Changes made to the file, in file order.
    pub revisions: Vec<Revision>,
}

/// A header value with its OSIS `type` attribute.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Typed<K> {
    /// `None` when the attribute is missing or not one we know.
    pub kind: Option<K>,
    pub text: RawSpan,
}

/// An enum parsed from an OSIS `type` attribute.
trait Kind: Sized {
    fn from_attr(value: &[u8]) -> Option<Self>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageKind {
    /// Ethnologue code, e.g. `eng`.
    Ethnologue,
    /// Name in English, e.g. `17th century England dialect of English`.
    InEnglish,
    /// Name in the language itself, e.g. `English`.
    Vernacular,
}

impl Kind for LanguageKind {
    fn from_attr(value: &[u8]) -> Option<Self> {
        match value {
            b"x-ethnologue" => Some(Self::Ethnologue),
            b"x-in-english" => Some(Self::InEnglish),
            b"x-vernacular" => Some(Self::Vernacular),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublisherKind {
    /// e.g. `Cambridge University Press, Oxford University Press, Collins, etc.`.
    Print,
    /// e.g. `eBible.org`.
    Electronic,
}

impl Kind for PublisherKind {
    fn from_attr(value: &[u8]) -> Option<Self> {
        match value {
            b"x-print" => Some(Self::Print),
            b"x-electronic" => Some(Self::Electronic),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightsKind {
    /// e.g. `public domain`.
    Copyright,
    /// The full license text.
    License,
}

impl Kind for RightsKind {
    fn from_attr(value: &[u8]) -> Option<Self> {
        match value {
            b"x-copyright" => Some(Self::Copyright),
            b"x-license" => Some(Self::License),
            _ => None,
        }
    }
}

/// An entry in the file's change log.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Revision {
    /// Free form, e.g. `2013.07.08`.
    pub date: RawSpan,
    /// What changed, e.g. `created or updated OSIS file contents`.
    pub description: RawSpan,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Book {
    /// Short name, e.g. `Genesis`.
    pub name: RawSpan,
    /// Full title, e.g. `The First Book of Moses, called Genesis`.
    pub title: RawSpan,
    /// Title of the group the book is in, e.g. `Old Testament`.
    pub group: RawSpan,
    pub chapters: Vec<Chapter>,
}

impl Book {
    pub fn get_chapters(&self) -> impl Iterator<Item = &Chapter> {
        self.chapters.iter()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
#[allow(dead_code)]
pub struct Chapter {
    pub number: usize,
    /// e.g. `Gen.1`.
    pub osis_ref: RawSpan,
    pub verses: Vec<Verse>,
}

impl Chapter {
    pub fn get_verses(&self) -> impl Iterator<Item = &Verse> {
        self.verses.iter()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
#[allow(dead_code)]
pub struct Verse {
    pub number: usize,
    /// e.g. `Gen.1.1`.
    pub osis_id: RawSpan,
    /// Text before the verse that is not part of any verse, e.g. psalm titles.
    pub headings: Vec<Heading>,
    /// Inline markup splits the text into multiple segments.
    pub text: Vec<Segment>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Heading {
    pub kind: HeadingKind,
    pub text: Vec<Segment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingKind {
    /// Canonical psalm title, e.g. `A Psalm of David.`.
    Psalm,
    /// Non canonical title, e.g. `The Prologue of the Wisdom of Jesus the Son of Sirach.`.
    Sub,
    /// Division of a book, e.g. `Book I` of the Psalms.
    Division,
    /// Prose outside of any verse, e.g. the Sirach prologue.
    Paragraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Segment {
    Text {
        span: RawSpan,
        /// Added by the translators, KJV prints these in italics.
        added: bool,
        /// Red letter text.
        words_of_jesus: bool,
    },
    /// Start of a paragraph.
    Paragraph,
    /// Start of a poetry line.
    Line,
    /// Break between stanzas.
    Stanza,
}

impl Heading {
    fn new(kind: HeadingKind) -> Self {
        Self {
            kind,
            text: Vec::new(),
        }
    }
}

/// What a `<title>` names, when it is not a heading.
#[derive(Clone, Copy)]
enum TitleOf {
    Group,
    Book,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Bible {
    pub fn from_translation(translation: &str) -> Result<Bible> {
        let compressed = get_translations()
            // TODO: Select translation from UI.
            .get("KJV")
            .ok_or(Error::UnsupprtedTranslation(translation.to_string()))?;
        let start = Instant::now();
        let raw = decompress_xz(compressed)?;
        info!("Decompressed {translation} in {:?}", start.elapsed());
        Bible::from_raw(raw)
    }

    /// For translations that are not built in.
    #[allow(dead_code)]
    pub fn from_file(path: &std::path::Path) -> Result<Bible> {
        info!("Loading {:?} into memory", path);
        let start = Instant::now();
        let raw = if is_xz_compressed_xml(path) {
            decompress_xz(&std::fs::read(path)?)?
        } else if is_xml_file(path) {
            std::fs::read_to_string(path)?
        } else {
            return Err(Error::InvalidBibleFile);
        };
        info!("Loaded {:?} in {:?}", path, start.elapsed());
        Bible::from_raw(raw)
    }

    /// `raw` is the OSIS XML.
    fn from_raw(raw: String) -> Result<Bible> {
        if raw.len() > u32::MAX as usize {
            return Err(Error::InvalidBibleFile);
        }
        let metadata = Self::parse_metadata(&raw)?;
        let books = Self::build_index_from_osis(&raw)?;
        return Ok(Bible {
            metadata,
            books,
            raw: raw,
        });
    }

    // Accessors.

    /// The exact bytes from the file.
    pub fn raw(&self, span: RawSpan) -> &str {
        &self.raw[span.start as usize..span.end as usize]
    }

    /// Display text: entities unescaped and whitespace collapsed.
    pub fn text(&self, segments: &[Segment]) -> String {
        // There seems to be some groups of whitespaces in the data.  I guess the file is just
        // dirty.
        let s: String = segments
            .iter()
            .map(|segment| match *segment {
                Segment::Text { span, .. } => self.raw(span),
                Segment::Paragraph | Segment::Line | Segment::Stanza => " ",
            })
            .collect();
        let s = quick_xml::escape::unescape(&s).unwrap_or(s.as_str().into());
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[allow(dead_code)]
    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Book names under their group title, e.g. `Old Testament`, in file order.
    pub fn book_groups(&self) -> Vec<(String, Vec<String>)> {
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        for book in &self.books {
            let group = self.raw(book.group);
            let name = self.raw(book.name).to_string();
            match groups.last_mut() {
                Some((title, names)) if title == group => names.push(name),
                _ => groups.push((group.to_string(), vec![name])),
            }
        }
        groups
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
        let mut capture: Option<(&[u8], &[u8], usize)> = None;

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
                    let kind = Self::attr_bytes(raw, e, b"type");
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
        kind: &[u8],
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
            b"language" => metadata.languages.push(Self::typed(kind, text)),
            b"publisher" => metadata.publishers.push(Self::typed(kind, text)),
            b"rights" => metadata.rights.push(Self::typed(kind, text)),
            _ => {}
        }
    }

    fn build_index_from_osis(raw: &str) -> Result<Vec<Book>> {
        info!("Building bible index");

        let start = Instant::now();
        let mut books: Vec<Book> = Vec::new();
        let mut reader = Reader::from_str(raw);

        let mut awaiting_group_title = false;
        let mut awaiting_book_title = false;
        let mut group = RawSpan::default();
        // What the next `</title>` belongs to, and the start of its content.
        let mut title: Option<(TitleOf, usize)> = None;
        // The heading being read, and those waiting for the next verse.
        let mut heading: Option<Heading> = None;
        let mut headings: Vec<Heading> = Vec::new();
        // Breaks waiting for the next verse.
        let mut breaks: Vec<Segment> = Vec::new();

        let mut in_verse = false;
        let mut added = false;
        let mut words_of_jesus = false;
        // Used when an element has no `n`.
        let mut current_chapter = 1;
        let mut current_verse = 1;

        loop {
            let before = reader.buffer_position() as usize;
            let event = reader
                .read_event()
                .map_err(|e| Error::BibleIndex(e.to_string()))?;
            let after = reader.buffer_position() as usize;

            match event {
                Event::Start(ref e) if e.name().as_ref() == b"div" => {
                    awaiting_group_title = Self::has_attr_val(e, b"type", b"bookGroup");
                    awaiting_book_title = Self::has_attr_val(e, b"type", b"book");
                }
                Event::Start(ref e) if e.name().as_ref() == b"title" && awaiting_group_title => {
                    title = Some((TitleOf::Group, after));
                    awaiting_group_title = false;
                }
                Event::Start(ref e) if e.name().as_ref() == b"title" && awaiting_book_title => {
                    if let Some(name) = Self::attr_span(raw, e, b"short") {
                        books.push(Book {
                            name,
                            group,
                            ..Default::default()
                        });
                        title = Some((TitleOf::Book, after));
                    }
                    awaiting_book_title = false;
                    current_chapter = 1;
                }
                Event::Start(ref e) if e.name().as_ref() == b"title" && !in_verse => {
                    let kind = match Self::attr_bytes(raw, e, b"type") {
                        b"psalm" => Some(HeadingKind::Psalm),
                        b"sub" => Some(HeadingKind::Sub),
                        _ => None,
                    };
                    heading = kind.map(Heading::new);
                }
                Event::End(ref e) if e.name().as_ref() == b"title" => match title.take() {
                    Some((TitleOf::Group, start)) => group = RawSpan::new(start, before),
                    Some((TitleOf::Book, start)) => {
                        if let Some(book) = books.last_mut() {
                            book.title = RawSpan::new(start, before);
                        }
                    }
                    None => headings.extend(heading.take()),
                },
                Event::Start(ref e) if e.name().as_ref() == b"p" => {
                    if in_verse {
                        Self::push_break(&mut books, &mut breaks, in_verse, Segment::Paragraph);
                    } else if Self::has_attr_val(e, b"type", b"x-ms") {
                        heading = Some(Heading::new(HeadingKind::Division));
                    } else {
                        // Prose, unless a verse starts before any text.
                        heading = Some(Heading::new(HeadingKind::Paragraph));
                    }
                }
                Event::End(ref e) if e.name().as_ref() == b"p" => {
                    headings.extend(heading.take().filter(|h| !h.text.is_empty()));
                }
                Event::Start(ref e) if e.name().as_ref() == b"l" => {
                    Self::push_break(&mut books, &mut breaks, in_verse, Segment::Line);
                }
                Event::Empty(ref e) if e.name().as_ref() == b"lb" => {
                    Self::push_break(&mut books, &mut breaks, in_verse, Segment::Stanza);
                }
                Event::Start(ref e) if e.name().as_ref() == b"transChange" => {
                    added = Self::has_attr_val(e, b"type", b"added");
                }
                Event::End(ref e) if e.name().as_ref() == b"transChange" => added = false,
                // Quotes are milestones, `eID` ends the quote.
                Event::Empty(ref e) if e.name().as_ref() == b"q" => {
                    words_of_jesus =
                        Self::has_attr(e, b"sID") && Self::has_attr_val(e, b"who", b"Jesus");
                }
                Event::Empty(ref e)
                    if e.name().as_ref() == b"chapter" && Self::has_attr(e, b"sID") =>
                {
                    current_chapter = Self::number(e).unwrap_or(current_chapter);
                    if let Some(book) = books.last_mut() {
                        book.chapters.push(Chapter {
                            number: current_chapter,
                            osis_ref: Self::attr_span(raw, e, b"osisRef").unwrap_or_default(),
                            verses: Vec::new(),
                        });
                    }
                    current_chapter += 1;
                    current_verse = 1;
                }
                Event::Empty(ref e)
                    if e.name().as_ref() == b"verse" && Self::has_attr(e, b"sID") =>
                {
                    assert!(!in_verse);
                    in_verse = true;
                    match heading.take() {
                        Some(h) if h.kind == HeadingKind::Paragraph && h.text.is_empty() => {
                            breaks.push(Segment::Paragraph)
                        }
                        h => headings.extend(h),
                    }
                    current_verse = Self::number(e).unwrap_or(current_verse);
                    let verse = Verse {
                        number: current_verse,
                        osis_id: Self::attr_span(raw, e, b"osisID").unwrap_or_default(),
                        headings: std::mem::take(&mut headings),
                        text: std::mem::take(&mut breaks),
                    };
                    if let Some(ch) = books.last_mut().and_then(|b| b.chapters.last_mut()) {
                        ch.verses.push(verse);
                    }
                    current_verse += 1;
                }
                Event::Empty(ref e)
                    if e.name().as_ref() == b"verse" && Self::has_attr(e, b"eID") =>
                {
                    assert!(in_verse);
                    in_verse = false;
                }
                // Entities arrive as their own event, keep them escaped in the span.
                Event::Text(_) | Event::GeneralRef(_) if after > before => {
                    let segment = Segment::Text {
                        span: RawSpan::new(before, after),
                        added,
                        words_of_jesus,
                    };
                    if in_verse {
                        if let Some(verse) = Self::last_verse(&mut books) {
                            verse.text.push(segment);
                        }
                    } else if let Some(h) = &mut heading {
                        // Skip the indentation before the first word.
                        if !h.text.is_empty() || !raw[before..after].trim().is_empty() {
                            h.text.push(segment);
                        }
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

    fn typed<K: Kind>(kind: &[u8], text: RawSpan) -> Typed<K> {
        Typed {
            kind: K::from_attr(kind),
            text,
        }
    }

    fn last_verse(books: &mut [Book]) -> Option<&mut Verse> {
        books.last_mut()?.chapters.last_mut()?.verses.last_mut()
    }

    /// Breaks outside of a verse belong to the next one.
    fn push_break(books: &mut [Book], breaks: &mut Vec<Segment>, in_verse: bool, b: Segment) {
        match in_verse {
            true => Self::last_verse(books).into_iter().for_each(|v| v.text.push(b)),
            false => breaks.push(b),
        }
    }

    /// The `n` attribute of a chapter or verse.
    fn number(e: &BytesStart) -> Option<usize> {
        let a = e
            .attributes()
            .filter_map(|a| a.ok())
            .find(|a| a.key.as_ref() == b"n")?;
        std::str::from_utf8(&a.value).ok()?.parse().ok()
    }

    /// Attribute value, empty if missing.
    fn attr_bytes<'a>(raw: &'a str, e: &BytesStart, key: &[u8]) -> &'a [u8] {
        Self::attr_span(raw, e, key)
            .map(|s| &raw.as_bytes()[s.start as usize..s.end as usize])
            .unwrap_or_default()
    }

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
