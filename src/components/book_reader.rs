use crate::app::events::{AppEvent, UserAction};
use crate::bible::Bible;
use crate::components::Component;
use crate::prelude::*;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Stylize;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

pub struct BookReader {
    bible: Arc<Bible>,
    current_book_name: String,
    scrolled_offset: usize,
    focused: bool,
    book_changed: bool,
    view_dirty: bool,
    num_columns: usize,
}

impl BookReader {
    pub fn new(bible: Arc<Bible>, current_book_name: String) -> Self {
        BookReader {
            bible,
            current_book_name,
            scrolled_offset: 0,
            focused: false,
            book_changed: true,
            view_dirty: true,
            // TODO: Change based on terminal size.
            num_columns: 2,
        }
    }

    pub fn set_book(&mut self, book: &str) {
        if self.current_book_name != book {
            self.current_book_name = book.to_string();
            self.scrolled_offset = 0;
            self.book_changed = true;
        }
    }

    /// To simulate scrolling the pages we use the layout.  It switches from N to 2N "columns"
    /// depending on the scroll.
    fn layout(inner: Rect, scrolled_offset: usize, num_columns: usize) -> Vec<Rect> {
        let page_height = inner.height as usize;
        let phase = scrolled_offset % (page_height.max(1));

        let hcols = |row: Rect| {
            let mut constraints = Vec::with_capacity(num_columns * 2 + 1);
            for i in 0..num_columns {
                constraints.push(Constraint::Length(2));
                constraints.push(Constraint::Fill(1));
                if i == num_columns - 1 {
                    constraints.push(Constraint::Length(2));
                }
            }
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(row);
            (0..num_columns)
                .map(|i| cols[i * 2 + 1])
                .collect::<Vec<_>>()
        };

        if phase == 0 {
            hcols(inner)
        } else {
            let vertical_gap = 2;
            let shrink_amount = phase as u16;
            let top_height = inner
                .height
                .saturating_sub(shrink_amount)
                .saturating_sub(vertical_gap);
            let bottom_height = inner.height - top_height - vertical_gap;

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(top_height),
                    Constraint::Length(vertical_gap),
                    Constraint::Length(bottom_height),
                ])
                .split(inner);

            vec![rows[0], rows[2]]
                .iter()
                .flat_map(|row| hcols(*row))
                .collect()
        }
    }

    /// Calculates how many chars can fit into a single column.
    fn chars_per_column(area: Rect, num_columns: usize) -> usize {
        let inner_width = area.width.saturating_sub(2);
        let inner_height = area.height.saturating_sub(2);
        let col_width = inner_width / num_columns.max(1) as u16;
        (col_width as usize) * (inner_height as usize)
    }

    fn column_text(bible: &Bible, book: &str, char_limit: usize) -> Vec<Span<'static>> {
        let Ok(book_index) = bible.get_book_index(book) else {
            return vec![];
        };

        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut char_count = 0;

        'outer: for (ch_idx, chapter) in book_index.chapters.iter().enumerate() {
            let chapter_num = ch_idx + 1;
            for (v_idx, _) in chapter.verses.iter().enumerate() {
                let verse_num = v_idx + 1;
                let text = bible
                    .get_verse_iter(book, chapter_num, verse_num)
                    .map(|it| it.collect::<Vec<_>>().join(" "))
                    .unwrap_or_default();

                let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
                let num_str = format!("{} ", verse_num);
                let body_str = format!("{} ", normalized);

                char_count += num_str.len() + body_str.len();
                spans.push(Span::styled(num_str, Style::default().dark_gray()));
                spans.push(Span::raw(body_str));

                if char_count >= char_limit {
                    break 'outer;
                }
            }
        }

        spans
    }
}

impl Component for BookReader {
    fn update(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Focus => self.focused = true,
            AppEvent::Defocus => self.focused = false,
            AppEvent::UserAction(action) if self.focused => match action {
                UserAction::MoveDown => {
                    self.scrolled_offset = self.scrolled_offset.saturating_add(1);
                    self.view_dirty = true;
                }
                UserAction::MoveUp => {
                    self.scrolled_offset = self.scrolled_offset.saturating_sub(1);
                    self.view_dirty = true;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!(" [2] {} ", self.current_book_name).yellow().bold())
            .border_style(if self.focused {
                Style::default().blue()
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        block.render(area, buf);

        let char_limit = Self::chars_per_column(area, self.num_columns);
        let spans = Self::column_text(&self.bible, &self.current_book_name, char_limit);
        let mk = || Paragraph::new(Line::from(spans.clone())).wrap(Wrap { trim: false });

        for pane in Self::layout(inner, self.scrolled_offset, self.num_columns) {
            mk().render(pane, buf);
        }
    }
}
