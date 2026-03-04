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
        overflow: Option<ColumnChapter>,
    ) -> (Column, Option<ColumnChapter>) {
        let mut remaining_budget = width * height;
        let mut chapters: Vec<ColumnChapter> = Vec::new();

        // TODO: Handle.
        assert!(overflow.is_none());

        let remainder = None;
        loop {
            let chapter = ColumnChapter::from_chapter_naive(bible, start_chapter, width);
            let (fit, remainder) = chapter.split(width, remaining_budget);

            match fit {
                None => {
                    assert!(remainder.is_some());
                    break;
                }

                Some(fit) => {
                    let fit_consumed = fit.consumed_chars(width);
                    debug!("consumed; {fit_consumed}");
                    debug!("budhet: {remaining_budget}");
                    chapters.push(fit);
                    assert!(fit_consumed <= remaining_budget); // Fit can't be larger than the budet.
                    remaining_budget -= fit_consumed;
                    remaining_budget = remaining_budget.saturating_sub(width); // Newline.
                }
            }

            // If we had to split, or there is no budget left, time to stop.
            if remainder.is_some() || remaining_budget == 0 {
                break;
            }
        }

        let column = Column {
            width,
            height,
            chapters,
        };
        (column, remainder)
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

        // TODO: Build
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
                match first {
                    None => break,
                    Some(first) => column_verses.push(first),
                };
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

        let padding = width - (verse_cost % width);
        header + verse_cost + padding
    }

    /// TODO: Doc.
    pub fn split(
        self,
        width: usize,
        budget: usize,
    ) -> (Option<ColumnChapter>, Option<ColumnChapter>) {
        // It does not makes sense for us to get a budget that is not `N * rows`.
        assert!(budget % width == 0);
        // Budget of 0 also does not makes sense.
        assert!(budget > 0);

        // If chapter fits into budget no split will occur.
        if self.consumed_chars(width) <= budget {
            return (Some(self), None);
        }
        // Need to be able to fit at least the header and a single verse.
        if budget < (width * 2) {
            return (None, Some(self));
        }

        let verses_clone = self.verses.clone(); // TODO: Fix.
        let mut num_fitting_verses = 0;
        let mut current_row_offset = 0;
        let mut first_split: Option<ColumnVerseSegment> = None;
        let mut second_split: Option<ColumnVerseSegment> = None;

        let mut remaining_budget = match self.show_heading {
            false => budget,
            true => budget - width,
        };

        for verse in self.verses {
            let verse_chars = verse.consumed_chars(width, current_row_offset);

            // Verse fits, no issues.
            if verse_chars <= remaining_budget {
                num_fitting_verses += 1;
                current_row_offset = (current_row_offset + verse_chars) % width;
                remaining_budget -= verse_chars;
                continue;
            }

            debug!("verse chars: {verse_chars}");
            debug!("remaining budget: {remaining_budget}");

            // Verse did not fit, have to split.
            (first_split, second_split) = verse.split_at_wrap(width, current_row_offset);
            break;
        }

        let (first, second) = verses_clone.split_at(num_fitting_verses);
        let mut first = first.to_vec();
        debug!("{first:?}");
        let mut second = second.to_vec();

        if let Some(split) = first_split {
            first.push(split);
        }
        debug!("{first:?}");
        if let Some(split) = second_split {
            debug!("before: {:?}", second[0]);
            second[0] = split;
            debug!("after: {:?}", second[0]);
        }

        // TODO: I don't like the clones here.  Should be &str.
        let to_chapter =
            |verses: Vec<ColumnVerseSegment>, show_heading: bool| -> Option<ColumnChapter> {
                (!verses.is_empty()).then(|| ColumnChapter {
                    show_heading,
                    number: self.number,
                    verses,
                })
            };
        (to_chapter(first, true), to_chapter(second, false))
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
            text: text.trim_end().trim_start().to_string(),
        }
    }

    /// Calculates the verse rows crated by wrapping.  Each vec is a row.
    ///
    /// Returns:
    /// - The consumed chars for the row (including numbers and whitespace)
    /// - The raw index that can be used to index into the existing string.
    ///
    /// WARN: This function is nasty.
    pub fn calculate_verse_rows_from_wrapping(
        &self,
        width: usize,
        starting_offset: usize,
    ) -> Vec<(usize, usize)> {
        // TODO: Tweak reserving.
        let mut rows = Vec::new();
        rows.reserve(10);

        let mut current_string_index = 0; // Used to index into memory.
        let mut remaining_chars_in_row = width - starting_offset;
        let mut current_row_raw_size = 0;

        // This is 0 when the number won't be showed so no need for extra checking.
        // TODO: Currently this assumes the verse number can be split from the first word.
        // Decide what we want and revisit.
        let number_size = self.get_number_char_size();
        // Number does not fit, we need to leave it empty and wrap.
        if number_size > remaining_chars_in_row {
            rows.push((remaining_chars_in_row, 0));
            remaining_chars_in_row = width - number_size;
            current_row_raw_size = 0;
        }
        // Number does fit.
        // Number not part of raw text, so don't increase `current_raw_text_size`.
        else {
            current_row_raw_size += number_size;
            remaining_chars_in_row -= number_size;
        }

        // Now check words.
        let mut first_word_in_row = starting_offset == 0;
        self.text.split_whitespace().for_each(|word| {
            let word_len = word.len();

            // The first word in the row does not need a leading whitespace,
            let word_consumption = if first_word_in_row {
                word_len
            } else {
                word_len + 1
            };

            // Check if word fits (no wrap).
            if word_consumption <= remaining_chars_in_row {
                current_row_raw_size += word_consumption;
                remaining_chars_in_row -= word_consumption;
                current_string_index += word_consumption;
                first_word_in_row = false;
            }
            // Wrap.
            else {
                // Consume trailing whitespace for previous word.
                current_row_raw_size += remaining_chars_in_row;
                rows.push((current_row_raw_size, current_string_index));

                // Reset, going to a new line.
                remaining_chars_in_row = width - word_len;

                // Usage on next line.
                current_row_raw_size = word_len; // No leading whitespace.
                current_string_index = word_consumption;
            }
        });

        // Push last consumption.  Do not use trailing whitespace, the next verse can fit.
        rows.push((current_row_raw_size, current_string_index));

        rows
    }

    /// This includes leading whitespace used by the verse, and not trailing whitespace.
    pub fn consumed_chars(&self, width: usize, starting_offset: usize) -> usize {
        self.calculate_verse_rows_from_wrapping(width, starting_offset)
            .iter()
            .map(|(a, _)| a)
            .sum::<usize>()
    }

    /// Splits the verse where it wraps, returning the remainder.
    ///
    /// NOTE: The second segment is not guaranteed to fit a row, check manually.
    /// NOTE: `character_budget` includes whitespace, i.e. it is the rendering budget.
    fn split_at_wrap(
        self,
        width: usize,
        starting_offset: usize,
    ) -> (Option<ColumnVerseSegment>, Option<ColumnVerseSegment>) {
        let mut character_budget = width - starting_offset;
        let mut accumulated_index = 0;

        let regions = self.calculate_verse_rows_from_wrapping(width, starting_offset);

        // No verses in first segment.
        if regions.first().expect("There should be a region here").1 == 0 {
            return (None, Some(self));
        }

        // Find point where we have to split.
        for (consumed, raw_index) in &regions {
            if *consumed <= character_budget {
                character_budget -= consumed;
                accumulated_index += raw_index;
            } else {
                break;
            }
        }

        // Trimming here just to clean up.
        let first = self.text[..accumulated_index].trim_end().to_string();
        let second = self.text[accumulated_index..].trim_start().to_string();

        let remainder = match second.is_empty() {
            true => None,
            false => Some(ColumnVerseSegment {
                show_number: false,
                number: self.number,
                text: second,
            }),
        };

        (
            Some(ColumnVerseSegment {
                show_number: self.show_number,
                number: self.number,
                text: first,
            }),
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
    fn get_number_char_size(&self) -> usize {
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
