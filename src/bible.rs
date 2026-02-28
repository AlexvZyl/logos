use crate::{
    filesystem::{decompress_xz, is_xml_file, is_xz_compressed_xml},
    prelude::*,
};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::time::Instant;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Bible {
    translation: String,
    disk_file: PathBuf,

    index: HashMap<String, Book>,
    raw: String,
}

#[derive(Default, Debug)]
pub struct Book {
    pub chapters: Vec<Chapter>,
}

impl Book {
    pub fn get_num_chapters(&self) -> usize {
        self.chapters.len()
    }
}

#[derive(Default, Debug)]
pub struct Chapter {
    pub verses: Vec<VerseView>,
}

impl Chapter {
    pub fn get_num_verses(&self) -> usize {
        self.verses.len()
    }
}

/// A non-owning view into the raw memory.
#[derive(Default, Debug)]
pub struct VerseView {
    /// (start, end) byte ranges of text regions in the raw data.
    pub text: Vec<(usize, usize)>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Bible {
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

        return Ok(Bible {
            disk_file: path.to_path_buf(),
            index: Self::build_index(&raw)?,
            translation: "KJV".to_string(), // TODO: Get translation.
            raw: raw,
        });
    }

    pub fn get_books(&self) -> Vec<&String> {
        self.index.keys().collect()
    }

    pub fn get_book_index(&self, name: &str) -> Result<&Book> {
        self.index
            .get(name)
            .ok_or(Error::BookNotFound(name.to_string()))
    }

    pub fn get_chapter_index(&self, name: &str, chapter: usize) -> Result<&Chapter> {
        if chapter == 0 {
            return Err(Error::ChapterNotFound(name.to_string(), chapter));
        }

        self.get_book_index(name)?
            .chapters
            .get(chapter - 1)
            .ok_or(Error::ChapterNotFound(name.to_string(), chapter))
    }

    pub fn get_verse_index(&self, name: &str, chapter: usize, verse: usize) -> Result<&VerseView> {
        if chapter == 0 {
            return Err(Error::VerseNotFound(name.to_string(), chapter, verse));
        }

        self.get_chapter_index(name, chapter)?
            .verses
            .get(verse - 1)
            .ok_or(Error::VerseNotFound(name.to_string(), chapter, verse))
    }

    pub fn get_verse_iter(
        &self,
        name: &str,
        chapter: usize,
        verse: usize,
    ) -> Result<impl Iterator<Item = &str>> {
        let v = self.get_verse_index(name, chapter, verse)?;
        let raw = &self.raw;
        Ok(v.text.iter().map(move |&(s, e)| &raw[s..e]))
    }

    fn build_index(raw: &str) -> Result<HashMap<String, Book>> {
        info!("Building bible index");
        let start = Instant::now();
        let mut index: HashMap<String, Book> = HashMap::new();
        let mut reader = Reader::from_str(raw);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut book = String::new();
        let mut awaiting_title = false;
        let mut in_verse = false;

        loop {
            let offset = reader.buffer_position() as usize;
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
                }
                Ok(Event::Empty(ref e))
                    if e.name().as_ref() == b"chapter" && Self::has_attr(e, b"sID") =>
                {
                    index
                        .entry(book.clone())
                        .or_default()
                        .chapters
                        .push(Chapter::default());
                }
                Ok(Event::Empty(ref e))
                    if e.name().as_ref() == b"verse" && Self::has_attr(e, b"sID") =>
                {
                    in_verse = true;
                    if let Some(ch) = index.get_mut(&book).and_then(|b| b.chapters.last_mut()) {
                        ch.verses.push(VerseView::default());
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
                        if let Some(verse) = index
                            .get_mut(&book)
                            .and_then(|b| b.chapters.last_mut())
                            .and_then(|c| c.verses.last_mut())
                        {
                            verse.text.push((offset, offset + len));
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

    // Parsing utils.

    fn attr(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<String> {
        e.attributes()
            .filter_map(|a| a.ok())
            .find(|a| a.key.as_ref() == key)
            .map(|a| String::from_utf8_lossy(&a.value).to_string())
    }

    fn has_attr(e: &quick_xml::events::BytesStart, key: &[u8]) -> bool {
        e.attributes()
            .filter_map(|a| a.ok())
            .any(|a| a.key.as_ref() == key)
    }

    fn has_attr_val(e: &quick_xml::events::BytesStart, key: &[u8], val: &[u8]) -> bool {
        e.attributes()
            .filter_map(|a| a.ok())
            .any(|a| a.key.as_ref() == key && a.value.as_ref() == val)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
