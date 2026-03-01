pub mod book_reader;
pub mod books_view;
pub mod footer;
pub mod references;
pub mod splash_screen;
pub mod strongs;

use crate::app::events::AppEvent;
use crate::prelude::*;

pub trait Component {
    fn update(&mut self, event: &AppEvent);
    fn render(&mut self, area: Rect, buf: &mut Buffer);
}
