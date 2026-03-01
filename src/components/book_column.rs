use crate::{bible::Bible, components::Component, prelude::*};

////////////////////////////////////////////////////////////////////////////////////////////////////

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

        for (chapter_num, chapter_data) in book_index.chapters_from(start_chapter) {
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
                number: chapter_num,
                verses: Vec::new(),
            };
            let chapter_cost = current_chapter.consumed_chars(width);
            if chapter_cost >= remaining_chars {
                return Ok((current_column, None));
            }
            remaining_chars -= chapter_cost;

            let mut last_verse_num = 0;
            for (verse_num, text) in chapter_data.verses_from(1, bible.raw()) {
                let current_verse = ColumnVerse {
                    show_number: verse_num != last_verse_num,
                    number: verse_num,
                    text: text.to_string(),
                };
                last_verse_num = verse_num;
                let verse_cost = current_verse.consumed_chars(width);

                // Entire verse does not fit.
                if verse_cost > remaining_chars {
                    let (first, second) = current_verse.split(width, remaining_chars);
                    current_chapter.verses.push(first);
                    current_column.chapters.push(current_chapter);

                    let overflow = match second {
                        None => None,
                        Some(verse) => Some(Column {
                            width: width,
                            height: height,
                            chapters: vec![ColumnChapter {
                                show_heading: false,
                                number: chapter_num,
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

pub struct ColumnChapter {
    pub show_heading: bool,
    pub number: usize,
    pub verses: Vec<ColumnVerse>,
}

impl ColumnChapter {
    pub fn consumed_chars(&self, width: usize) -> usize {
        let header = if self.show_heading { width } else { 0 };
        let verse_cost: usize = self.verses.iter().map(|v| v.consumed_chars(width)).sum();

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
            let cost = verse.consumed_chars(width);
            if cost <= budget {
                budget -= cost;
                first.push(verse);
            } else {
                let (a, b) = verse.split(width, budget);
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

pub struct ColumnVerse {
    pub show_number: bool,
    pub number: usize,
    pub text: String,
}

impl ColumnVerse {
    /// Walks over the words in the verse and handles the wrapping logic.
    ///
    /// Iterator gets (verse word, chars consumed by the word).
    ///
    /// If a word wraps, it will be seen as consuming the whitespace in the previous line.
    pub fn walk_words(&self, width: usize) -> impl Iterator<Item = (&str, usize)> {
        let initial = if self.show_number { 3 } else { 0 };

        self.text.split_whitespace().scan(
            (initial, width - initial),
            move |(consumed, remaining), w| {
                let prev = *consumed;
                let space = if *remaining < width { 1 } else { 0 };
                let needed = w.len() + space;

                // No wrap.
                if needed <= *remaining {
                    *consumed += needed;
                    *remaining -= needed;
                // Wrap.
                } else {
                    *consumed += *remaining + w.len();
                    *remaining = width - w.len();
                }

                Some((w, *consumed - prev))
            },
        )
    }

    pub fn consumed_chars(&self, width: usize) -> usize {
        let initial = if self.show_number { 3 } else { 0 };
        initial + self.walk_words(width).map(|(_, c)| c).sum::<usize>()
    }

    pub fn split(self, width: usize, budget: usize) -> (ColumnVerse, Option<ColumnVerse>) {
        let initial = if self.show_number { 3 } else { 0 };
        let mut consumed = initial;
        let mut split_byte = 0;

        for (word, cost) in self.walk_words(width) {
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////
