use std::time::Instant;

use crate::{
    filesystem::{decompress_xz, is_xml_file, is_xz_compressed_xml},
    prelude::*,
};
use quick_xml::Reader;
use quick_xml::events::Event;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Bible {
    translation: String,
    disk_file: PathBuf,

    pub index: HashMap<String, Book>,
    raw: String,
}

#[derive(Default, Debug)]
pub struct Book {
    pub chapters: Vec<Chapter>,
}

#[derive(Default, Debug)]
pub struct Chapter {
    pub verses: Vec<Verse>,
}

#[derive(Default, Debug)]
pub struct Verse {
    /// Byte offset in the raw data.
    pub offset: usize,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Bible {
    pub fn from_file(path: &std::path::Path) -> Result<Bible> {
        let raw = if is_xz_compressed_xml(path) {
            decompress_xz(path)?
        } else if is_xml_file(path) {
            std::fs::read_to_string(path)?
        } else {
            return Err(Error::InvalidBibleFile);
        };

        return Ok(Bible {
            disk_file: path.to_path_buf(),
            index: Self::build_index(&raw)?,
            translation: "KJV".to_string(), // TODO: Get translation.
            raw: raw,
        });
    }

    fn build_index(raw: &str) -> Result<HashMap<String, Book>> {
        info!("Building bible index");
        let start = Instant::now();
        let mut index: HashMap<String, Book> = HashMap::new();
        let mut reader = Reader::from_str(raw);
        reader.config_mut().trim_text(true);

        let mut buffer = Vec::new();
        let mut current_book = String::new();
        let mut current_chapter: Option<usize> = None;
        let mut awaiting_book_title = false;

        loop {
            let offset = reader.buffer_position();
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"div" => {
                        let is_book = e
                            .attributes()
                            .filter_map(|a| a.ok())
                            .any(|a| a.key.as_ref() == b"type" && a.value.as_ref() == b"book");
                        if is_book {
                            awaiting_book_title = true;
                        }
                    }
                    b"title" if awaiting_book_title => {
                        if let Some(short) = e
                            .attributes()
                            .filter_map(|a| a.ok())
                            .find(|a| a.key.as_ref() == b"short")
                        {
                            current_book = String::from_utf8_lossy(&short.value).to_string();
                            index.entry(current_book.clone()).or_default();
                        }
                        awaiting_book_title = false;
                    }
                    _ => {}
                },
                Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                    b"chapter" => {
                        let is_start = e
                            .attributes()
                            .filter_map(|a| a.ok())
                            .any(|a| a.key.as_ref() == b"sID");
                        if is_start {
                            if let Some(n) = e
                                .attributes()
                                .filter_map(|a| a.ok())
                                .find(|a| a.key.as_ref() == b"n")
                            {
                                let chapter_num: usize =
                                    String::from_utf8_lossy(&n.value).parse().unwrap_or(0);
                                current_chapter = Some(chapter_num);
                                let book = index.entry(current_book.clone()).or_default();
                                book.chapters.push(Chapter::default());
                                debug!("Chapter {} in {}", chapter_num, current_book);
                            }
                        } else {
                            current_chapter = None;
                        }
                    }
                    b"verse" => {
                        let is_start = e
                            .attributes()
                            .filter_map(|a| a.ok())
                            .any(|a| a.key.as_ref() == b"sID");
                        if is_start {
                            if let Some(book) = index.get_mut(&current_book) {
                                if let Some(chapter) = book.chapters.last_mut() {
                                    chapter.verses.push(Verse {
                                        offset: offset as usize,
                                    });
                                }
                            }
                            if let Some(osis_id) = e
                                .attributes()
                                .filter_map(|a| a.ok())
                                .find(|a| a.key.as_ref() == b"osisID")
                            {
                                let osis_id = String::from_utf8_lossy(&osis_id.value);
                                debug!("Verse: {}", osis_id);
                            }
                        }
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::BibleIndex(e.to_string())),
                _ => {}
            }
            buffer.clear();
        }

        info!("Built index in {:?}", start.elapsed());
        Ok(index)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
