//! Cell representation for terminal buffer.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Text style attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Style {
    /// Foreground color
    pub fg: Option<Color>,
    /// Background color
    pub bg: Option<Color>,
    /// Bold text
    pub bold: bool,
    /// Dim/faint text
    pub dim: bool,
    /// Italic text
    pub italic: bool,
    /// Underlined text
    pub underline: bool,
    /// Blinking text
    pub blink: bool,
    /// Reverse video (swap fg/bg)
    pub reverse: bool,
    /// Hidden text
    pub hidden: bool,
    /// Strikethrough text
    pub strikethrough: bool,
}

impl Style {
    /// Create a new style with default values.
    #[inline]
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            blink: false,
            reverse: false,
            hidden: false,
            strikethrough: false,
        }
    }

    /// Set foreground color.
    #[inline]
    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set background color.
    #[inline]
    pub const fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Enable bold.
    #[inline]
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Enable dim.
    #[inline]
    pub const fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Enable italic.
    #[inline]
    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Enable underline.
    #[inline]
    pub const fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Merge another style on top of this one.
    /// Non-None values from `other` override `self`.
    #[inline]
    pub fn merge(&self, other: &Style) -> Self {
        Self {
            fg: other.fg.or(self.fg),
            bg: other.bg.or(self.bg),
            bold: other.bold || self.bold,
            dim: other.dim || self.dim,
            italic: other.italic || self.italic,
            underline: other.underline || self.underline,
            blink: other.blink || self.blink,
            reverse: other.reverse || self.reverse,
            hidden: other.hidden || self.hidden,
            strikethrough: other.strikethrough || self.strikethrough,
        }
    }
}

/// Terminal color representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    /// Reset to default
    Reset,
    /// Black (ANSI 0)
    Black,
    /// Red (ANSI 1)
    Red,
    /// Green (ANSI 2)
    Green,
    /// Yellow (ANSI 3)
    Yellow,
    /// Blue (ANSI 4)
    Blue,
    /// Magenta (ANSI 5)
    Magenta,
    /// Cyan (ANSI 6)
    Cyan,
    /// White (ANSI 7)
    White,
    /// Bright black / Gray (ANSI 8)
    Gray,
    /// Bright red (ANSI 9)
    LightRed,
    /// Bright green (ANSI 10)
    LightGreen,
    /// Bright yellow (ANSI 11)
    LightYellow,
    /// Bright blue (ANSI 12)
    LightBlue,
    /// Bright magenta (ANSI 13)
    LightMagenta,
    /// Bright cyan (ANSI 14)
    LightCyan,
    /// Bright white (ANSI 15)
    LightWhite,
    /// 256-color palette (0-255)
    Indexed(u8),
    /// RGB true color
    Rgb(u8, u8, u8),
}

impl Color {
    /// Create RGB color from hex string (e.g., "#ff0000" or "ff0000").
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    }
}

impl From<Color> for crossterm::style::Color {
    fn from(color: Color) -> Self {
        match color {
            Color::Reset => crossterm::style::Color::Reset,
            Color::Black => crossterm::style::Color::Black,
            Color::Red => crossterm::style::Color::DarkRed,
            Color::Green => crossterm::style::Color::DarkGreen,
            Color::Yellow => crossterm::style::Color::DarkYellow,
            Color::Blue => crossterm::style::Color::DarkBlue,
            Color::Magenta => crossterm::style::Color::DarkMagenta,
            Color::Cyan => crossterm::style::Color::DarkCyan,
            Color::White => crossterm::style::Color::Grey,
            Color::Gray => crossterm::style::Color::DarkGrey,
            Color::LightRed => crossterm::style::Color::Red,
            Color::LightGreen => crossterm::style::Color::Green,
            Color::LightYellow => crossterm::style::Color::Yellow,
            Color::LightBlue => crossterm::style::Color::Blue,
            Color::LightMagenta => crossterm::style::Color::Magenta,
            Color::LightCyan => crossterm::style::Color::Cyan,
            Color::LightWhite => crossterm::style::Color::White,
            Color::Indexed(i) => crossterm::style::Color::AnsiValue(i),
            Color::Rgb(r, g, b) => crossterm::style::Color::Rgb { r, g, b },
        }
    }
}

/// A single cell in the terminal buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The character(s) displayed in this cell.
    /// May be empty (space) or contain multi-byte Unicode.
    pub symbol: CompactString,
    /// The style of this cell.
    pub style: Style,
    /// Whether this is a wide character continuation cell.
    /// Wide characters (e.g., CJK) span 2 cells.
    pub is_continuation: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Cell {
    /// Empty cell constant (space with default style).
    pub const EMPTY: Self = Self {
        symbol: CompactString::const_new(" "),
        style: Style::new(),
        is_continuation: false,
    };

    /// Create a new cell with the given character.
    #[inline]
    pub fn new(symbol: impl Into<CompactString>) -> Self {
        Self {
            symbol: symbol.into(),
            style: Style::new(),
            is_continuation: false,
        }
    }

    /// Create a new cell with character and style.
    #[inline]
    pub fn with_style(symbol: impl Into<CompactString>, style: Style) -> Self {
        Self {
            symbol: symbol.into(),
            style,
            is_continuation: false,
        }
    }

    /// Set the symbol.
    #[inline]
    pub fn set_symbol(&mut self, symbol: impl Into<CompactString>) {
        self.symbol = symbol.into();
        self.is_continuation = false;
    }

    /// Set the style.
    #[inline]
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    /// Mark as continuation cell (for wide characters).
    #[inline]
    pub fn set_continuation(&mut self) {
        self.symbol = CompactString::default();
        self.is_continuation = true;
    }

    /// Reset to empty space.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::EMPTY;
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, Color, Style};

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert_eq!(cell.symbol.as_str(), " ");
        assert!(!cell.is_continuation);
    }

    #[test]
    fn test_cell_with_style() {
        let style = Style::new().fg(Color::Red).bold();
        let cell = Cell::with_style("A", style);
        assert_eq!(cell.symbol.as_str(), "A");
        assert_eq!(cell.style.fg, Some(Color::Red));
        assert!(cell.style.bold);
    }

    #[test]
    fn test_color_from_hex() {
        assert_eq!(Color::from_hex("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(Color::from_hex("00ff00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(Color::from_hex("invalid"), None);
    }

    #[test]
    fn test_style_merge() {
        let base = Style::new().fg(Color::Red);
        let overlay = Style::new().bg(Color::Blue).bold();
        let merged = base.merge(&overlay);
        assert_eq!(merged.fg, Some(Color::Red));
        assert_eq!(merged.bg, Some(Color::Blue));
        assert!(merged.bold);
    }
}
