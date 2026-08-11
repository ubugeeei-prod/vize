//! Double-buffered terminal buffer.

use unicode_segmentation::UnicodeSegmentation;

use super::cell::{Cell, Style};
use crate::{layout::Rect, text::TextWidth};

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
    ///
    /// A grapheme intersecting the area is cleared in full, including a
    /// leading cell outside the area. This prevents partial wide glyphs from
    /// surviving a clipped repaint.
    pub fn clear_area(&mut self, area: Rect) {
        for y in area.y..area.y.saturating_add(area.height) {
            for x in area.x..area.x.saturating_add(area.width) {
                self.clear_grapheme_at(x, y);
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

    /// Set a raw cell at the given position.
    ///
    /// Prefer [`set_string`](Self::set_string) for visible text. This low-level
    /// operation intentionally does not repair adjacent continuation cells.
    #[inline]
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = cell;
        }
    }

    /// Set a character at the given position with optional style.
    pub fn set_char(&mut self, x: u16, y: u16, ch: char, style: Option<Style>) {
        let inherited_style = self.get(x, y).map_or_else(Style::new, |cell| cell.style);
        let mut encoded = [0_u8; 4];
        let text = ch.encode_utf8(&mut encoded);
        self.set_string(x, y, text, style.unwrap_or(inherited_style));
    }

    /// Set a string starting at the given position, preserving grapheme clusters.
    ///
    /// Each extended grapheme cluster is stored intact in its leading cell;
    /// any additional terminal columns are continuation cells. A cluster that
    /// cannot fit is clipped atomically, so no partial glyph is written.
    /// Returns the number of columns successfully written.
    pub fn set_string(&mut self, x: u16, y: u16, text: &str, style: Style) -> u16 {
        if y >= self.height || x >= self.width {
            return 0;
        }
        let mut col = x;
        if text.is_ascii() {
            for byte in text.bytes() {
                if byte.is_ascii_control() {
                    continue;
                }
                let mut encoded = [0_u8; 4];
                if !self.write_grapheme(
                    col,
                    y,
                    char::from(byte).encode_utf8(&mut encoded),
                    1,
                    style,
                ) {
                    break;
                }
                col += 1;
            }
        } else {
            for grapheme in text.graphemes(true) {
                let Ok(width) = u16::try_from(TextWidth::width(grapheme)) else {
                    break;
                };
                if width == 0 {
                    continue;
                }
                if !self.write_grapheme(col, y, grapheme, width, style) {
                    break;
                }
                col += width;
            }
        }
        col.saturating_sub(x)
    }

    fn write_grapheme(&mut self, x: u16, y: u16, grapheme: &str, width: u16, style: Style) -> bool {
        if width == 0 || y >= self.height || width > self.width.saturating_sub(x) {
            return false;
        }
        self.clear_span(x, y, width);
        let cell = self.get_mut(x, y).expect("validated grapheme origin");
        cell.set_symbol(grapheme);
        cell.set_style(style);
        for offset in 1..width {
            let continuation = self
                .get_mut(x + offset, y)
                .expect("validated grapheme continuation");
            continuation.set_continuation();
            continuation.set_style(style);
        }
        true
    }

    fn clear_span(&mut self, x: u16, y: u16, width: u16) {
        for column in x..x.saturating_add(width).min(self.width) {
            self.clear_grapheme_at(column, y);
        }
    }

    fn clear_grapheme_at(&mut self, x: u16, y: u16) {
        let Some(index) = self.index(x, y) else {
            return;
        };
        let row_start = usize::from(y) * usize::from(self.width);
        let row_end = row_start + usize::from(self.width);
        let mut leader = index;
        while leader > row_start && self.cells[leader].is_continuation {
            leader -= 1;
        }
        self.cells[leader].reset();
        let mut continuation = leader + 1;
        while continuation < row_end && self.cells[continuation].is_continuation {
            self.cells[continuation].reset();
            continuation += 1;
        }
    }

    /// Fill a rectangular area with a character.
    pub fn fill(&mut self, area: Rect, ch: char, style: Style) {
        self.clear_area(area);
        let width = TextWidth::char_width(ch) as u16;
        if width == 0 {
            return;
        }
        let mut encoded = [0_u8; 4];
        let grapheme = ch.encode_utf8(&mut encoded);
        for y in area.y..area.y.saturating_add(area.height) {
            let end = area.x.saturating_add(area.width).min(self.width);
            let mut x = area.x;
            while width <= end.saturating_sub(x) {
                if !self.write_grapheme(x, y, grapheme, width, style) {
                    break;
                }
                x += width;
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
    ///
    /// Complete source graphemes are copied atomically. A wide grapheme that
    /// crosses the destination's right edge is skipped rather than truncated.
    pub fn merge(&mut self, other: &Buffer, x: u16, y: u16) {
        for oy in 0..other.height {
            let Some(destination_y) = y.checked_add(oy).filter(|row| *row < self.height) else {
                continue;
            };
            let mut ox = 0;
            while ox < other.width {
                let cell = other.get(ox, oy).expect("source coordinate is in bounds");
                if cell.is_continuation {
                    ox += 1;
                    continue;
                }
                let mut span = 1;
                while ox + span < other.width
                    && other
                        .get(ox + span, oy)
                        .is_some_and(|next| next.is_continuation)
                {
                    span += 1;
                }
                let Some(destination_x) = x.checked_add(ox) else {
                    break;
                };
                if span <= self.width.saturating_sub(destination_x) {
                    self.clear_span(destination_x, destination_y, span);
                    for offset in 0..span {
                        let source = other
                            .get(ox + offset, oy)
                            .expect("validated source grapheme span");
                        self.set(destination_x + offset, destination_y, source.clone());
                    }
                }
                ox += span;
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
#[path = "buffer_tests.rs"]
mod tests;
