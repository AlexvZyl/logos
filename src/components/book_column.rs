use crate::{bible::Bible, components::Component, prelude::*};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Column {
    pub width: usize,
    pub height: usize,
    pub chapters: Vec<ColumnChapter>,
}

impl Column {
    pub fn new(width: usize, height: usize) -> Column {
        Column {
            width,
            height,
            chapters: Vec::new(),
        }
    }

    pub fn from(
        width: usize,
        height: usize,
        bible: &Bible,
        book: &str,
        start_chapter: usize,
        overflow: Option<Column>,
    ) -> Result<(Column, Option<Column>)> {
        let mut current_column = overflow.unwrap_or(Column::new(width, height));
        let mut remaining_chars = width * height - current_column.chars_consumed();

        let book_index = bible.get_book_index(book)?;

        for chapter in book_index.get_chapters().skip(start_chapter - 1) {
            let separator = if current_column.chapters.is_empty() {
                0
            } else {
                width
            };
            if separator >= remaining_chars {
                return Ok((current_column, None));
            }
            remaining_chars -= separator;

            let mut current_chapter = ColumnChapter {
                show_heading: true,
                number: chapter.number,
                verses: Vec::new(),
            };
            let chapter_cost = current_chapter.consumed_chars(width);
            if chapter_cost >= remaining_chars {
                return Ok((current_column, None));
            }
            remaining_chars -= chapter_cost;

            let mut last_verse_num = 0;
            for verse in chapter.get_verses() {
                let current_verse = ColumnVerse {
                    show_number: verse.number != last_verse_num,
                    number: verse.number,
                    text: verse.to_string(bible.get_raw_data()),
                };
                last_verse_num = verse.number;
                let verse_cost = current_verse.consumed_chars(width, 0);

                // Entire verse does not fit.
                if verse_cost > remaining_chars {
                    let (first, second) = current_verse.split(width, 0, remaining_chars);
                    current_chapter.verses.push(first);
                    current_column.chapters.push(current_chapter);

                    let overflow = match second {
                        None => None,
                        Some(verse) => Some(Column {
                            width: width,
                            height: height,
                            chapters: vec![ColumnChapter {
                                show_heading: false,
                                number: chapter.number,
                                verses: vec![verse],
                            }],
                        }),
                    };
                    return Ok((current_column, overflow));
                }
                // Verse does fit.
                else {
                    remaining_chars -= verse_cost;
                    current_chapter.verses.push(current_verse);
                }
            }

            current_column.chapters.push(current_chapter);
        }

        Ok((current_column, None))
    }

    pub fn chars_consumed(&self) -> usize {
        self.chapters
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let separator = if i > 0 && c.show_heading {
                    self.width // Newline added after chapters.
                } else {
                    0
                };
                separator + c.consumed_chars(self.width)
            })
            .sum()
    }
}

impl Component for Column {
    fn update(&mut self, _event: &AppEvent) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Result<()> {
        let mut lines: Vec<Line> = Vec::new();

        for (i, chapter) in self.chapters.iter().enumerate() {
            if chapter.show_heading {
                if i > 0 {
                    lines.push(Line::raw(""));
                }
                lines.push(Line::styled(
                    format!("Chapter {}", chapter.number),
                    Style::default().italic().blue(),
                ));
            }
            let mut spans: Vec<Span> = Vec::new();
            for verse in &chapter.verses {
                if verse.show_number {
                    spans.push(Span::styled(
                        format!("{} ", verse.number),
                        Style::default().dark_gray(),
                    ));
                }
                spans.push(Span::raw(format!("{} ", verse.text.trim_start())));
            }
            if !spans.is_empty() {
                lines.push(Line::from(spans));
            }
        }

        Paragraph::new(lines)
            .wrap(ratatui::widgets::Wrap { trim: false })
            .render(area, buf);

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ColumnChapter {
    pub show_heading: bool,
    pub number: usize,
    pub verses: Vec<ColumnVerse>,
}

impl ColumnChapter {
    pub fn consumed_chars(&self, width: usize) -> usize {
        let header = if self.show_heading { width } else { 0 };
        let verse_cost: usize = self.verses.iter().map(|v| v.consumed_chars(width, 0)).sum();

        // The last verse will take the entire line.
        let remainder = match verse_cost % width {
            0 => 0,
            r => width - r,
        };

        header + verse_cost + remainder
    }

    pub fn split(self, width: usize, budget: usize) -> (ColumnChapter, Option<ColumnChapter>) {
        let mut budget = budget.saturating_sub(if self.show_heading { width } else { 0 });
        let mut first = Vec::new();
        let mut second_iter = self.verses.into_iter();

        for verse in second_iter.by_ref() {
            let cost = verse.consumed_chars(width, 0);
            if cost <= budget {
                budget -= cost;
                first.push(verse);
            } else {
                let (a, b) = verse.split(width, 0, budget);
                first.push(a);
                let second: Vec<_> = b.into_iter().chain(second_iter).collect();
                return (
                    ColumnChapter {
                        show_heading: self.show_heading,
                        number: self.number,
                        verses: first,
                    },
                    if second.is_empty() {
                        None
                    } else {
                        Some(ColumnChapter {
                            show_heading: false,
                            number: self.number,
                            verses: second,
                        })
                    },
                );
            }
        }

        (
            ColumnChapter {
                show_heading: self.show_heading,
                number: self.number,
                verses: first,
            },
            None,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ColumnVerse {
    pub show_number: bool,
    pub number: usize,
    pub text: String,
}

impl ColumnVerse {
    /// Walks over the words in the verse and handles the wrapping logic.
    ///
    /// Iterator gets:
    /// - verse word (no whitespace)
    /// - chars consumed by the word, including:
    ///     - whitespace for following word
    ///     - whitespace to fill wrapping
    pub fn walk_words(
        &self,
        width: usize,
        starting_offset: usize,
    ) -> impl Iterator<Item = (&str, usize)> {
        let remaining = width - starting_offset - self.get_number_offset();

        let nexts = self
            .text
            .split_whitespace()
            .skip(1)
            .map(Some)
            .chain(std::iter::once(None));

        self.text
            .split_whitespace()
            .zip(nexts)
            .scan(remaining, move |remaining, (w, next)| {
                if w.len() > *remaining {
                    let cost = *remaining + w.len();
                    *remaining = width - w.len();
                    Some((w, cost))
                } else {
                    *remaining -= w.len();
                    let trailing = match next {
                        None => 0,
                        Some(next) if next.len() + 1 > *remaining => *remaining,
                        Some(_) => 1,
                    };
                    *remaining -= trailing;
                    Some((w, w.len() + trailing))
                }
            })
    }

    pub fn consumed_chars(&self, width: usize, starting_offset: usize) -> usize {
        let words: Vec<_> = self.walk_words(width, starting_offset).collect();
        for (w, c) in &words {
            log::debug!("{:?} cost={}", w, c);
        }
        self.get_number_offset() + words.iter().map(|(_, c)| c).sum::<usize>()
    }

    pub fn split(
        self,
        width: usize,
        starting_offset: usize,
        budget: usize,
    ) -> (ColumnVerse, Option<ColumnVerse>) {
        let mut consumed = 0;
        let mut split_byte = 0;

        for (word, cost) in self.walk_words(width, starting_offset) {
            if consumed + cost > budget {
                break;
            }
            consumed += cost;
            split_byte = word.as_ptr() as usize - self.text.as_ptr() as usize + word.len();
        }

        let remainder = self.text[split_byte..].trim_start();
        (
            ColumnVerse {
                show_number: self.show_number,
                number: self.number,
                text: self.text[..split_byte].trim_end().to_string(),
            },
            if remainder.is_empty() {
                None
            } else {
                Some(ColumnVerse {
                    show_number: false,
                    number: self.number,
                    text: remainder.to_string(),
                })
            },
        )
    }

    fn get_number_offset(&self) -> usize {
        match self.show_number {
            false => 0,
            true if self.number == 1 => 2,
            true => {
                let digits = self.number.checked_ilog10().unwrap_or(0) as usize + 1;
                digits + 1
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
