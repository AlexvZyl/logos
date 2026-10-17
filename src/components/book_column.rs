use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    bible::{Bible, Book},
    prelude::*,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub const PAGE_GAP: usize = 5;

/// A stable location in a book, independent of wrapping.  Verse `0` is the chapter heading.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub chapter: usize,
    pub verse: usize,
}

/// A book pre-wrapped into rows of a fixed width.  Columns are slices of these rows.
#[derive(Debug)]
pub struct BookLayout {
    pub width: usize,
    pub rows: Vec<Line<'static>>,
    /// Position at the start of each row.
    pub positions: Vec<Position>,
}

impl BookLayout {
    pub fn new(bible: &Bible, book: &Book, width: usize) -> Self {
        let mut wrapper = Wrapper::new(width.max(1));

        for (i, chapter) in book.get_chapters().enumerate() {
            wrapper.set_position(Position {
                chapter: chapter.number,
                verse: 0,
            });
            if i > 0 {
                wrapper.push_row(Line::default());
            }
            wrapper.push_row(Line::styled(
                format!("Chapter {}", chapter.number),
                theme::CHAPTER_HEADING,
            ));

            for verse in chapter.get_verses() {
                wrapper.set_position(Position {
                    chapter: chapter.number,
                    verse: verse.number,
                });
                let text = verse.collect_string(bible.get_raw_data());
                let mut words = text.split_whitespace();
                let number = Span::styled(verse.number.to_string(), theme::VERSE_NUMBER);

                // Keep the verse number attached to the first word.
                match words.next() {
                    Some(first) => {
                        wrapper.push_unit(vec![number, Span::raw(" "), Span::raw(first.to_string())])
                    }
                    None => wrapper.push_unit(vec![number]),
                }
                words.for_each(|w| wrapper.push_unit(vec![Span::raw(w.to_string())]));
            }
            wrapper.flush();
        }

        BookLayout {
            width,
            rows: wrapper.rows,
            positions: wrapper.positions,
        }
    }

    /// Row containing the start of `position`.
    pub fn row_of(&self, position: Position) -> usize {
        self.positions.partition_point(|&p| p <= position).saturating_sub(1)
    }

    /// Position of the first text row visible in the first column.
    pub fn position_at(&self, offset: usize, num_columns: usize, height: usize) -> Position {
        let stride = height + PAGE_GAP;
        let page_row = offset / stride;
        let within = offset % stride;
        let row = match within < height {
            true => page_row * num_columns * height + within,
            false => (page_row + 1) * num_columns * height,
        };
        let last = self.positions.len().saturating_sub(1);
        self.positions.get(row.min(last)).copied().unwrap_or_default()
    }

    /// Offset that puts the row containing `position` at the top of its column.
    pub fn offset_of(&self, position: Position, num_columns: usize, height: usize) -> usize {
        if height == 0 {
            return 0;
        }
        let row = self.row_of(position);
        (row / height / num_columns.max(1)) * (height + PAGE_GAP) + row % height
    }

    /// Pages are `height` rows tall and laid out in a grid, `num_columns` pages per row, with
    /// `PAGE_GAP` blank rows between page rows.  `offset` scrolls the whole grid vertically.
    pub fn column(
        &self,
        index: usize,
        num_columns: usize,
        offset: usize,
        height: usize,
    ) -> Vec<Line<'static>> {
        let stride = height + PAGE_GAP;
        (offset..offset + height)
            .map(|r| {
                let within = r % stride;
                if within >= height {
                    return Line::default();
                }
                let page = (r / stride) * num_columns + index;
                self.rows.get(page * height + within).cloned().unwrap_or_default()
            })
            .collect()
    }

    /// Offset at which the last row of pages is aligned to the top.
    pub fn max_offset(&self, num_columns: usize, height: usize) -> usize {
        if height == 0 {
            return 0;
        }
        let pages = self.rows.len().div_ceil(height);
        pages.div_ceil(num_columns.max(1)).saturating_sub(1) * (height + PAGE_GAP)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Greedy word wrapper.  A unit is a group of spans that should not be separated.
struct Wrapper {
    width: usize,
    rows: Vec<Line<'static>>,
    positions: Vec<Position>,
    current: Vec<Span<'static>>,
    current_width: usize,
    position: Position,
    /// Position of the first span in `current`.
    row_start: Position,
}

impl Wrapper {
    fn new(width: usize) -> Self {
        Wrapper {
            width,
            rows: Vec::new(),
            positions: Vec::new(),
            current: Vec::new(),
            current_width: 0,
            position: Position::default(),
            row_start: Position::default(),
        }
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
        if self.current.is_empty() {
            self.row_start = position;
        }
    }

    /// Pushes a full row, flushing any pending content first.
    fn push_row(&mut self, line: Line<'static>) {
        self.flush();
        self.rows.push(line);
        self.positions.push(self.position);
    }

    fn push_unit(&mut self, unit: Vec<Span<'static>>) {
        let unit_width: usize = unit.iter().map(|s| s.content.width()).sum();
        let separator = usize::from(!self.current.is_empty());

        if self.current_width + separator + unit_width > self.width {
            self.flush();
        }

        if unit_width > self.width {
            unit.into_iter().for_each(|s| self.push_hard(s));
            return;
        }

        if !self.current.is_empty() {
            self.current.push(Span::raw(" "));
            self.current_width += 1;
        }
        self.current_width += unit_width;
        self.current.extend(unit);
    }

    /// Splits a span by characters, for units wider than a full row.
    fn push_hard(&mut self, span: Span<'static>) {
        let mut chunk = String::new();
        for c in span.content.chars() {
            let w = c.width().unwrap_or(0);
            if self.current_width + w > self.width {
                if !chunk.is_empty() {
                    self.current
                        .push(Span::styled(std::mem::take(&mut chunk), span.style));
                }
                self.flush();
            }
            chunk.push(c);
            self.current_width += w;
        }
        if !chunk.is_empty() {
            self.current.push(Span::styled(chunk, span.style));
        }
    }

    fn flush(&mut self) {
        if !self.current.is_empty() {
            self.rows.push(Line::from(std::mem::take(&mut self.current)));
            self.positions.push(self.row_start);
        }
        self.current_width = 0;
        self.row_start = self.position;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
