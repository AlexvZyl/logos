//! Every non-ASCII character drawn by the app.

/// Nerd font icons.
///
/// TODO: Fall back to plain unicode when nerd fonts are not available.
#[derive(Debug, Clone, Copy)]
pub enum Icon {
    /// Before the app name in the footer.
    App,
    /// Marks the selected book.
    Selected,
}

impl Icon {
    pub const fn as_str(self) -> &'static str {
        match self {
            Icon::App => "\u{eebe}",
            Icon::Selected => "\u{f0da}",
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Box drawing and block characters.
#[derive(Debug, Clone, Copy)]
pub enum Symbol {
    /// Separates sections in the help overlay.
    HorizontalLine,
    ScrollbarThumb,
}

impl Symbol {
    pub const fn as_str(self) -> &'static str {
        match self {
            Symbol::HorizontalLine => "─",
            Symbol::ScrollbarThumb => "▌",
        }
    }
}
