//! Cursor management for terminal.

use serde::{Deserialize, Serialize};

/// Cursor position in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Cursor {
    /// Column (0-indexed)
    pub x: u16,
    /// Row (0-indexed)
    pub y: u16,
    /// Whether cursor is visible
    pub visible: bool,
    /// Cursor shape
    pub shape: CursorShape,
    /// Whether cursor is blinking
    pub blinking: bool,
}

impl Cursor {
    /// Create a new cursor at origin.
    #[inline]
    pub const fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            visible: true,
            shape: CursorShape::Block,
            blinking: true,
        }
    }

    /// Create cursor at specific position.
    #[inline]
    pub const fn at(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            visible: true,
            shape: CursorShape::Block,
            blinking: true,
        }
    }

    /// Move cursor to position.
    #[inline]
    pub fn move_to(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    /// Move cursor by delta.
    #[inline]
    pub fn move_by(&mut self, dx: i16, dy: i16) {
        self.x = (self.x as i16 + dx).max(0) as u16;
        self.y = (self.y as i16 + dy).max(0) as u16;
    }

    /// Move cursor right by n columns.
    #[inline]
    pub fn move_right(&mut self, n: u16) {
        self.x = self.x.saturating_add(n);
    }

    /// Move cursor left by n columns.
    #[inline]
    pub fn move_left(&mut self, n: u16) {
        self.x = self.x.saturating_sub(n);
    }

    /// Move cursor down by n rows.
    #[inline]
    pub fn move_down(&mut self, n: u16) {
        self.y = self.y.saturating_add(n);
    }

    /// Move cursor up by n rows.
    #[inline]
    pub fn move_up(&mut self, n: u16) {
        self.y = self.y.saturating_sub(n);
    }

    /// Show the cursor.
    #[inline]
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the cursor.
    #[inline]
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Set cursor shape.
    #[inline]
    pub fn set_shape(&mut self, shape: CursorShape) {
        self.shape = shape;
    }

    /// Set cursor blinking.
    #[inline]
    pub fn set_blinking(&mut self, blinking: bool) {
        self.blinking = blinking;
    }
}

/// Cursor shape variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CursorShape {
    /// Block cursor (default)
    #[default]
    Block,
    /// Underline cursor
    Underline,
    /// Vertical bar cursor
    Bar,
}

impl CursorShape {
    /// Convert to crossterm SetCursorStyle command.
    pub fn to_cursor_style(&self) -> crossterm::cursor::SetCursorStyle {
        match self {
            CursorShape::Block => crossterm::cursor::SetCursorStyle::SteadyBlock,
            CursorShape::Underline => crossterm::cursor::SetCursorStyle::SteadyUnderScore,
            CursorShape::Bar => crossterm::cursor::SetCursorStyle::SteadyBar,
        }
    }

    /// Convert to blinking crossterm SetCursorStyle command.
    pub fn to_blinking_cursor_style(&self) -> crossterm::cursor::SetCursorStyle {
        match self {
            CursorShape::Block => crossterm::cursor::SetCursorStyle::BlinkingBlock,
            CursorShape::Underline => crossterm::cursor::SetCursorStyle::BlinkingUnderScore,
            CursorShape::Bar => crossterm::cursor::SetCursorStyle::BlinkingBar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cursor;

    #[test]
    fn test_cursor_new() {
        let cursor = Cursor::new();
        assert_eq!(cursor.x, 0);
        assert_eq!(cursor.y, 0);
        assert!(cursor.visible);
    }

    #[test]
    fn test_cursor_move() {
        let mut cursor = Cursor::new();
        cursor.move_to(5, 10);
        assert_eq!(cursor.x, 5);
        assert_eq!(cursor.y, 10);

        cursor.move_by(3, -5);
        assert_eq!(cursor.x, 8);
        assert_eq!(cursor.y, 5);
    }

    #[test]
    fn test_cursor_move_saturating() {
        let mut cursor = Cursor::at(2, 2);
        cursor.move_left(10);
        assert_eq!(cursor.x, 0);

        cursor.move_up(10);
        assert_eq!(cursor.y, 0);
    }
}
