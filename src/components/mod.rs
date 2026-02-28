pub mod book_reader;
pub mod books_view;
pub mod footer;
pub mod splash_screen;

use crate::app::events::AppEvent;
use crate::prelude::*;

pub trait Component {
    fn update(&mut self, event: &AppEvent);
    fn render(&mut self, area: Rect, buf: &mut Buffer);
}
