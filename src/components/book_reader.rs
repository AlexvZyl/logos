use crate::app::events::{AppEvent, UserAction};
use crate::bible::Bible;
use crate::components::Component;
use crate::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

pub struct BookReader {
    pub scroll_offset: u16,
    focused: bool,
    book_name: String,
    cached_lines: Vec<Line<'static>>,
}

impl BookReader {
    pub fn new(bible: &Bible, book: &str) -> Self {
        let cached_lines = Self::build_lines(bible, book);
        BookReader {
            scroll_offset: 0,
            focused: false,
            book_name: book.to_string(),
            cached_lines,
        }
    }

    pub fn set_book(&mut self, bible: &Bible, book: &str) {
        if self.book_name != book {
            self.book_name = book.to_string();
            self.cached_lines = Self::build_lines(bible, book);
            self.scroll_offset = 0;
        }
    }

    fn build_lines(bible: &Bible, book: &str) -> Vec<Line<'static>> {
        let Ok(book_index) = bible.get_book_index(book) else {
            return vec![];
        };

        let mut lines: Vec<Line<'static>> = Vec::new();
        for (chapter_idx, chapter) in book_index.chapters.iter().enumerate() {
            let chapter_num = chapter_idx + 1;
            lines.push(
                Line::from(format!("Chapter {}", chapter_num))
                    .italic()
                    .light_blue(),
            );

            let mut spans: Vec<Span<'static>> = Vec::new();
            for (verse_idx, _) in chapter.verses.iter().enumerate() {
                let verse_num = verse_idx + 1;
                let text = bible
                    .get_verse_iter(book, chapter_num, verse_num)
                    .map(|it| it.collect::<Vec<_>>().join(" "))
                    .unwrap_or_default();

                spans.push(Span::raw(format!("{} ", verse_num)).dark_gray());
                spans.push(Span::raw(format!("{} ", text)));
            }
            lines.push(Line::from(spans));
            lines.push(Line::raw(""));
        }

        lines
    }
}

impl Component for BookReader {
    fn update(&mut self, event: &AppEvent) {
        match event {
            AppEvent::Focus => self.focused = true,
            AppEvent::Defocus => self.focused = false,
            AppEvent::UserAction(action) if self.focused => match action {
                UserAction::MoveDown => self.scroll_offset = self.scroll_offset.saturating_add(1),
                UserAction::MoveUp => self.scroll_offset = self.scroll_offset.saturating_sub(1),
                _ => {}
            },
            _ => {}
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!(" {} ", self.book_name).yellow().bold())
            .border_style(if self.focused {
                Style::default().blue()
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        block.render(area, buf);

        Paragraph::new(Text::from(self.cached_lines.clone()))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0))
            .render(inner, buf);
    }
}
