pub use crate::app::events::AppEvent;
pub use crate::config::*;
pub use crate::error::{Error, Result};

pub use log::{debug, error, info, trace, warn};
pub use std::collections::HashMap;
pub use std::path::{Path, PathBuf};
pub use std::time::Instant;

pub use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
