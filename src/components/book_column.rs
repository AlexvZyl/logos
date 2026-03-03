use crate::{
    bible::{Bible, Chapter},
    components::Component,
    prelude::*,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Column {
    pub width: usize,
    pub height: usize,
    pub chapters: Vec<ColumnChapter>,
}

impl Column {
    pub fn new(
        width: usize,
        height: usize,
        bible: &Bible,
        start_chapter: &Chapter,
        overflow: Option<Column>,
    ) -> (Column, Option<Column>) {
        // TODO: Handle this case.
        assert!(overflow.is_none());

        let chapter = ColumnChapter::from_chapter_naive(bible, start_chapter, width);

        (
            Column {
                width,
                height,
                chapters: Vec::from([chapter]),
            },
            None,
        )
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
            if i > 0 {
                lines.push(Line::raw(""));
            }
            lines.extend(chapter.build());
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
    pub verses: Vec<ColumnVerseSegment>,
}

impl ColumnChapter {
    /// Does not check splitting for the chapter, but does split verses.
    pub fn from_chapter_naive(bible: &Bible, chapter: &Chapter, width: usize) -> Self {
        // TODO: Tweak reserve.
        let mut column_verses = Vec::new();
        column_verses.reserve(10);

        // Create all of the verses, splitting as we go.
        let current_offset = 0;
        chapter.get_verses().for_each(|verse| {
            let mut remainder = ColumnVerseSegment::new_naive(
                verse.number,
                &verse.collect_string(bible.get_raw_data()),
            );

            // Need to split as we go.
            loop {
                let (first, next) = remainder.split_at_wrap(width, current_offset);
                debug!("{first:?}");
                column_verses.push(first);
                match next {
                    None => break,
                    Some(r) => remainder = r,
                }
            }
        });

        ColumnChapter {
            show_heading: true,
            number: chapter.number,
            verses: column_verses,
        }
    }

    // Does not include the potential gap before chapter.
    pub fn consumed_chars(&self, width: usize) -> usize {
        let header = if self.show_heading { width } else { 0 };
        let verse_cost: usize = self.verses.iter().map(|v| v.consumed_chars(width, 0)).sum();

        // The last verse will take the entire line.
        let total = header + verse_cost + (verse_cost % width);
        assert!(total % width == 0);
        total
    }

    pub fn split(self, width: usize, budget: usize) -> (ColumnChapter, Option<ColumnChapter>) {
        // It does not makes sense for us to get a budget that is not `N * rows`.
        assert!(width % budget == 0);
        // Budget of 0 also does not makes sense.
        assert!(budget > 0);

        // If chapter fits into budget no split will occur.
        if self.consumed_chars(width) < budget {
            return (self, None);
        }

        let number_verses = (budget - width) / width;
        // Should at least be able to render one verse.
        assert!(number_verses > 0);
        // TODO: There is a lot of assumptions made here that the render logic also has to make.
        // This separation makes me nervous.  Anyway...

        let (first, second) = self.verses.split_at(number_verses - 1);

        // TODO: I don't like the clones here.  Should be &str.
        (
            ColumnChapter {
                show_heading: true,
                number: self.number,
                verses: first.to_vec(),
            },
            Some(ColumnChapter {
                show_heading: false,
                number: self.number,
                verses: second.to_vec(),
            }),
        )
    }

    pub fn build(&self) -> Vec<Line> {
        assert!(!self.verses.is_empty());

        let mut lines: Vec<Line> = Vec::new();
        // TODO: Tweak reserve.
        lines.reserve(25);

        // Heading.
        if self.show_heading {
            lines.push(Line::styled(
                format!("Chapter {}", self.number),
                Style::default().italic().blue(),
            ));
        }

        // Verses.
        let mut spans: Vec<Span> = Vec::new();
        // TODO: Tweak reserve.
        spans.reserve(25);
        for verse in &self.verses {
            let (number, text) = verse.build();
            if let Some(n) = number {
                spans.push(n);
            }
            spans.push(text);
        }
        assert!(!spans.is_empty());
        lines.push(Line::from(spans));

        lines
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ColumnVerseSegment {
    pub show_number: bool,
    pub number: usize,
    // TODO: This should be &str.
    pub text: String,
}

impl ColumnVerseSegment {
    /// Creates a naive verse without any splitting.
    pub fn new_naive(number: usize, text: &str) -> Self {
        ColumnVerseSegment {
            show_number: true,
            number,
            text: text.to_string(),
        }
    }

    /// Returns:
    /// (Consumed region, raw text buffer size)
    ///
    /// The consumed region includes all of the whitespace required for rendering and wrapping of
    /// text.  The raw size can be used to index into the raw memory region.
    ///
    /// Each entry in the vec represent a line rendered.
    ///
    /// WARN: This function is nasty.
    pub fn get_consumption_regions(
        &self,
        width: usize,
        starting_offset: usize,
    ) -> Vec<(usize, usize)> {
        let mut consumptions = Vec::new();
        let mut current_raw_index_size = 0; // Used to index into memory.
        let mut remaining_chars_in_line = width - starting_offset;
        let mut current_consumption = 0;

        // This is 0 when the number won't be showed so no need for extra checking.
        // TODO: Currently this assumes the verse number can be split from the first word.
        // Decide what we want and revisit.
        let number_consumption = self.get_number_consumption();
        // Number does not fit (wraps).
        if number_consumption > remaining_chars_in_line {
            current_consumption += remaining_chars_in_line;
            remaining_chars_in_line = width;
        }
        // Number does fit.
        // Number not part of raw text, so don't increase `current_raw_text_size`.
        else {
            current_consumption += number_consumption;
            remaining_chars_in_line -= number_consumption;
        }

        // Now check words.
        let mut first_word_in_row = starting_offset == 0;
        self.text.split_whitespace().for_each(|word| {
            let word_len = word.len();

            // Words are separated by spaces in the raw data so need to be very careful with this
            // calculation.  The first word in the verse does not need a leading space, but the rest
            // do (if we have to show the number it means we are at the first word).
            let word_index_size = if self.show_number {
                word_len
            }
            // If it is not the first word we need to account for the leading space.
            else {
                word_len + 1
            };

            // The first word in the row does not need a leading whitespace,
            let word_consumption = if first_word_in_row {
                word_len
            } else {
                word_len + 1
            };

            // Check if word fits (no wrap).
            if word_consumption <= remaining_chars_in_line {
                current_consumption += word_consumption;
                remaining_chars_in_line -= word_consumption;

                current_raw_index_size += word_index_size;
                first_word_in_row = false;
            }
            // Wrap.
            else {
                // Consume trailing whitespace for previous word.
                current_consumption += remaining_chars_in_line;
                consumptions.push((current_consumption, current_raw_index_size));

                // Reset, going to a new line.
                remaining_chars_in_line = width;

                // Usage on next line.
                current_consumption = word_len; // No leading whitespace.
                current_raw_index_size = word_index_size;
            }
        });

        // Push last consumption.  Do not use trailing whitespace, the next verse can fit.
        consumptions.push((current_consumption, current_raw_index_size));

        consumptions
    }

    /// This includes leading whitespace used by the verse, and not trailing whitespace.
    pub fn consumed_chars(&self, width: usize, starting_offset: usize) -> usize {
        self.get_consumption_regions(width, starting_offset)
            .iter()
            .map(|(a, _)| a)
            .sum::<usize>()
    }

    /// Splits the verse where it wraps, returning the remainder.
    ///
    /// NOTE: `character_budget` includes whitespace, i.e. it is the rendering budget.
    fn split_at_wrap(
        self,
        width: usize,
        starting_offset: usize,
    ) -> (ColumnVerseSegment, Option<ColumnVerseSegment>) {
        let mut character_budget = width - starting_offset;
        let mut raw_index = 0;

        let regions = self.get_consumption_regions(width, starting_offset);
        // Find point where we have to split.
        regions.iter().for_each(|(consumed, raw_size)| {
            if *consumed <= character_budget {
                character_budget -= *consumed;
                raw_index += raw_size;
            }
        });

        // Trim is important here as this is assumed in `get_consumption_regions`.
        let first = self.text[..raw_index].trim_end().to_string();
        let second = self.text[raw_index..].trim_start().to_string();

        let remainder = match second.is_empty() {
            true => None,
            false => Some(ColumnVerseSegment {
                show_number: false,
                number: self.number,
                text: second,
            }),
        };

        (
            ColumnVerseSegment {
                show_number: self.show_number,
                number: self.number,
                text: first,
            },
            remainder,
        )
    }

    /// Returns:
    /// (verse number, verse text)
    ///
    /// TODO: TOO MANY CLONES!
    pub fn build(&self) -> (Option<Span>, Span) {
        assert!(!self.text.is_empty());

        let number = if self.show_number {
            // TODO: Cleanup
            if self.number == 1 {
                Some(Span::styled(
                    format!("{} ", self.number),
                    Style::default().dark_gray(),
                ))
            } else {
                Some(Span::styled(
                    format!(" {} ", self.number),
                    Style::default().dark_gray(),
                ))
            }
        } else {
            None
        };

        let text = Span::raw(self.text.to_string());
        (number, text)
    }

    // Includes leading whitespace.  Excludes trailing whitespace.
    fn get_number_consumption(&self) -> usize {
        match self.show_number {
            false => 0,
            true if self.number == 1 => 1,
            true => {
                // Includes leading whitespace.
                let digits = self.number.checked_ilog10().unwrap_or(0) as usize + 1;
                digits
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
