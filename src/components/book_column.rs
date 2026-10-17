use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    bible::{Bible, Book},
    prelude::*,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub const PAGE_GAP: usize = 5;

/// A book pre-wrapped into rows of a fixed width.  Columns are slices of these rows.
#[derive(Debug)]
pub struct BookLayout {
    pub width: usize,
    pub rows: Vec<Line<'static>>,
}

impl BookLayout {
    pub fn new(bible: &Bible, book: &Book, width: usize) -> Self {
        let mut wrapper = Wrapper::new(width.max(1));

        for (i, chapter) in book.get_chapters().enumerate() {
            if i > 0 {
                wrapper.rows.push(Line::default());
            }
            wrapper.rows.push(Line::styled(
                format!("Chapter {}", chapter.number),
                Style::default().italic().blue(),
            ));

            for verse in chapter.get_verses() {
                let text = verse.collect_string(bible.get_raw_data());
                let mut words = text.split_whitespace();
                let number = Span::styled(verse.number.to_string(), Style::default().dark_gray());

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
        }
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
    current: Vec<Span<'static>>,
    current_width: usize,
}

impl Wrapper {
    fn new(width: usize) -> Self {
        Wrapper {
            width,
            rows: Vec::new(),
            current: Vec::new(),
            current_width: 0,
        }
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
        }
        self.current_width = 0;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
