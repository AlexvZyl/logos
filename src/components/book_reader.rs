use crate::bible::Bible;
use crate::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

pub struct BookReader<'a> {
    pub bible: &'a Bible,
    pub book: &'a str,
    pub scroll_offset: u16,
    pub focused: bool,
}

impl Widget for BookReader<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!(" {} ", self.book).yellow().bold())
            .border_style(if self.focused {
                Style::default().blue()
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        block.render(area, buf);

        let Ok(book) = self.bible.get_book_index(self.book) else {
            return;
        };

        let mut lines: Vec<Line> = Vec::new();
        for (chapter_idx, chapter) in book.chapters.iter().enumerate() {
            let chapter_num = chapter_idx + 1;
            lines.push(
                Line::from(format!("Chapter {}", chapter_num))
                    .italic()
                    .light_blue(),
            );

            let mut spans: Vec<Span> = Vec::new();
            for (verse_idx, _) in chapter.verses.iter().enumerate() {
                let verse_num = verse_idx + 1;
                let text = self
                    .bible
                    .get_verse_iter(self.book, chapter_num, verse_num)
                    .map(|it| it.collect::<Vec<_>>().join(" "))
                    .unwrap_or_default();

                spans.push(Span::raw(format!("{} ", verse_num)).dark_gray());
                spans.push(Span::raw(format!("{} ", text)));
            }
            lines.push(Line::from(spans));

            lines.push(Line::raw(""));
        }

        Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0))
            .render(inner, buf);
    }
}
