//! Double-buffered terminal buffer.

use compact_str::ToCompactString;

use super::cell::{Cell, Style};
use crate::layout::Rect;

/// A buffer representing terminal content.
///
/// Uses a flat Vec for efficient memory layout and cache locality.
/// Supports double-buffering through swap operations.
#[derive(Debug, Clone)]
pub struct Buffer {
    /// Buffer cells stored in row-major order
    cells: Vec<Cell>,
    /// Buffer width
    width: u16,
    /// Buffer height
    height: u16,
}

impl Buffer {
    /// Create a new buffer with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            cells: vec![Cell::EMPTY; size],
            width,
            height,
        }
    }

    /// Get buffer width.
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Get buffer height.
    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Get the area covered by this buffer.
    #[inline]
    pub fn area(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    /// Resize the buffer, clearing all content.
    pub fn resize(&mut self, width: u16, height: u16) {
        let size = (width as usize) * (height as usize);
        self.cells.clear();
        self.cells.resize(size, Cell::EMPTY);
        self.width = width;
        self.height = height;
    }

    /// Clear the entire buffer.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.reset();
        }
    }

    /// Clear a specific area of the buffer.
    pub fn clear_area(&mut self, area: Rect) {
        for y in area.y..area.y.saturating_add(area.height) {
            for x in area.x..area.x.saturating_add(area.width) {
                if let Some(cell) = self.get_mut(x, y) {
                    cell.reset();
                }
            }
        }
    }

    /// Get index into cells vector from coordinates.
    #[inline]
    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y as usize) * (self.width as usize) + (x as usize))
        } else {
            None
        }
    }

    /// Get a cell at the given position.
    #[inline]
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        self.index(x, y).map(|i| &self.cells[i])
    }

    /// Get a mutable cell at the given position.
    #[inline]
    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        self.index(x, y).map(|i| &mut self.cells[i])
    }

    /// Set a cell at the given position.
    #[inline]
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = cell;
        }
    }

    /// Set a character at the given position with optional style.
    pub fn set_char(&mut self, x: u16, y: u16, ch: char, style: Option<Style>) {
        if let Some(cell) = self.get_mut(x, y) {
            cell.set_symbol(ch.to_compact_string());
            if let Some(s) = style {
                cell.set_style(s);
            }
        }
    }

    /// Set a string starting at the given position.
    /// Returns the number of columns used.
    pub fn set_string(&mut self, x: u16, y: u16, text: &str, style: Style) -> u16 {
        use unicode_width::UnicodeWidthChar;

        let mut col = x;
        for ch in text.chars() {
            if col >= self.width {
                break;
            }

            let width = ch.width().unwrap_or(0) as u16;
            if width == 0 {
                continue;
            }

            // Set the main character cell
            if let Some(cell) = self.get_mut(col, y) {
                cell.set_symbol(ch.to_compact_string());
                cell.set_style(style);
                cell.is_continuation = false;
            }

            // For wide characters, mark the next cell as continuation
            if width > 1 {
                for i in 1..width {
                    if let Some(cell) = self.get_mut(col + i, y) {
                        cell.set_continuation();
                        cell.set_style(style);
                    }
                }
            }

            col += width;
        }

        col.saturating_sub(x)
    }

    /// Fill a rectangular area with a character.
    pub fn fill(&mut self, area: Rect, ch: char, style: Style) {
        for y in area.y..area.y.saturating_add(area.height) {
            for x in area.x..area.x.saturating_add(area.width) {
                self.set_char(x, y, ch, Some(style));
            }
        }
    }

    /// Fill a rectangular area with a cell.
    pub fn fill_cell(&mut self, area: Rect, cell: Cell) {
        for y in area.y..area.y.saturating_add(area.height) {
            for x in area.x..area.x.saturating_add(area.width) {
                self.set(x, y, cell.clone());
            }
        }
    }

    /// Get an iterator over (x, y, cell) for all cells.
    pub fn iter(&self) -> impl Iterator<Item = (u16, u16, &Cell)> {
        self.cells.iter().enumerate().map(|(i, cell)| {
            let x = (i % self.width as usize) as u16;
            let y = (i / self.width as usize) as u16;
            (x, y, cell)
        })
    }

    /// Compute differences between this buffer and another.
    /// Returns an iterator of (x, y, cell) for cells that differ.
    pub fn diff<'a>(&'a self, other: &'a Buffer) -> impl Iterator<Item = (u16, u16, &'a Cell)> {
        self.cells
            .iter()
            .zip(other.cells.iter())
            .enumerate()
            .filter_map(move |(i, (a, b))| {
                if a != b {
                    let x = (i % self.width as usize) as u16;
                    let y = (i / self.width as usize) as u16;
                    Some((x, y, a))
                } else {
                    None
                }
            })
    }

    /// Merge another buffer onto this one at the specified position.
    pub fn merge(&mut self, other: &Buffer, x: u16, y: u16) {
        for oy in 0..other.height {
            for ox in 0..other.width {
                if let Some(cell) = other.get(ox, oy) {
                    self.set(x + ox, y + oy, cell.clone());
                }
            }
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;
    use crate::terminal::cell::{Cell, Style};

    #[test]
    fn test_buffer_new() {
        let buf = Buffer::new(80, 24);
        assert_eq!(buf.width(), 80);
        assert_eq!(buf.height(), 24);
    }

    #[test]
    fn test_buffer_set_get() {
        let mut buf = Buffer::new(10, 10);
        let cell = Cell::new("A");
        buf.set(5, 5, cell.clone());
        assert_eq!(buf.get(5, 5).map(|c| c.symbol.as_str()), Some("A"));
    }

    #[test]
    fn test_buffer_set_string() {
        let mut buf = Buffer::new(20, 1);
        let cols = buf.set_string(0, 0, "Hello", Style::new());
        assert_eq!(cols, 5);
        assert_eq!(buf.get(0, 0).map(|c| c.symbol.as_str()), Some("H"));
        assert_eq!(buf.get(4, 0).map(|c| c.symbol.as_str()), Some("o"));
    }

    #[test]
    fn test_buffer_wide_char() {
        let mut buf = Buffer::new(20, 1);
        let cols = buf.set_string(0, 0, "あ", Style::new()); // Wide char
        assert_eq!(cols, 2);
        assert_eq!(buf.get(0, 0).map(|c| c.symbol.as_str()), Some("あ"));
        assert!(buf.get(1, 0).map(|c| c.is_continuation).unwrap_or(false));
    }

    #[test]
    fn test_buffer_diff() {
        let mut buf1 = Buffer::new(5, 1);
        let mut buf2 = Buffer::new(5, 1);

        buf1.set_char(0, 0, 'A', None);
        buf2.set_char(0, 0, 'B', None);

        let diffs: Vec<_> = buf1.diff(&buf2).collect();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].0, 0);
        assert_eq!(diffs[0].1, 0);
    }

    #[test]
    fn test_buffer_resize() {
        let mut buf = Buffer::new(10, 10);
        buf.set_char(5, 5, 'X', None);
        buf.resize(20, 20);
        assert_eq!(buf.width(), 20);
        assert_eq!(buf.height(), 20);
        // Content should be cleared
        assert_eq!(buf.get(5, 5).map(|c| c.symbol.as_str()), Some(" "));
    }
}
