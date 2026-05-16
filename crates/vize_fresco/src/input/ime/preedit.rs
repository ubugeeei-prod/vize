//! Preedit (uncommitted text) handling.

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::text::{SegmentedText, TextWidth};
use vize_carton::cstr;

/// Preedit text with cursor and segments.
#[derive(Debug, Clone, Default)]
pub struct Preedit {
    /// The full preedit text
    text: CompactString,
    /// Cursor position (grapheme index)
    cursor: usize,
    /// Styled segments
    segments: SmallVec<[PreeditSegment; 8]>,
}

impl Preedit {
    /// Create empty preedit.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create preedit with text.
    pub fn with_text(text: impl Into<CompactString>) -> Self {
        let text = text.into();
        let len = SegmentedText::new(&text).grapheme_count;
        Self {
            text,
            cursor: len,
            segments: SmallVec::new(),
        }
    }

    /// Get the preedit text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get display width.
    pub fn width(&self) -> usize {
        TextWidth::width(&self.text)
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Get grapheme count.
    pub fn len(&self) -> usize {
        SegmentedText::new(&self.text).grapheme_count
    }

    /// Set the preedit text.
    pub fn set_text(&mut self, text: impl Into<CompactString>) {
        self.text = text.into();
        // Clamp cursor to valid range
        let max = self.len();
        if self.cursor > max {
            self.cursor = max;
        }
    }

    /// Set cursor position.
    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor.min(self.len());
    }

    /// Clear the preedit.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.segments.clear();
    }

    /// Get segments.
    pub fn segments(&self) -> &[PreeditSegment] {
        &self.segments
    }

    /// Set segments.
    pub fn set_segments(&mut self, segments: impl IntoIterator<Item = PreeditSegment>) {
        self.segments = segments.into_iter().collect();
    }

    /// Add a segment.
    pub fn add_segment(&mut self, segment: PreeditSegment) {
        self.segments.push(segment);
    }

    /// Get cursor column position (for display).
    pub fn cursor_column(&self) -> usize {
        SegmentedText::new(&self.text).column_at_index(self.cursor)
    }

    /// Insert text at cursor.
    pub fn insert(&mut self, text: &str) {
        let st = SegmentedText::new(&self.text);
        let byte_pos = if self.cursor >= st.grapheme_count {
            self.text.len()
        } else {
            // Find byte position for grapheme index
            self.text
                .char_indices()
                .filter_map(|(i, _)| {
                    let prefix = &self.text[..i];
                    let count = SegmentedText::new(prefix).grapheme_count;
                    if count == self.cursor {
                        Some(i)
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or(0)
        };

        let before = &self.text[..byte_pos];
        let after = &self.text[byte_pos..];
        self.text = cstr!("{before}{text}{after}");
        self.cursor += SegmentedText::new(text).grapheme_count;
    }

    /// Delete character before cursor.
    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let st = SegmentedText::new(&self.text);
        if self.cursor <= st.grapheme_count {
            // Get the text without the character before cursor
            let before = st.slice(0, self.cursor - 1);
            let after = st.slice(self.cursor, st.grapheme_count);
            self.text = cstr!("{before}{after}");
            self.cursor -= 1;
        }
    }

    /// Delete character at cursor.
    pub fn delete(&mut self) {
        let st = SegmentedText::new(&self.text);
        if self.cursor >= st.grapheme_count {
            return;
        }

        let before = st.slice(0, self.cursor);
        let after = st.slice(self.cursor + 1, st.grapheme_count);
        self.text = cstr!("{before}{after}");
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        if self.cursor < self.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start.
    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end.
    pub fn move_end(&mut self) {
        self.cursor = self.len();
    }
}

/// A segment of preedit text with styling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreeditSegment {
    /// Start index (grapheme)
    pub start: usize,
    /// End index (grapheme, exclusive)
    pub end: usize,
    /// Segment style
    pub style: SegmentStyle,
}

impl PreeditSegment {
    /// Create a new segment.
    pub fn new(start: usize, end: usize, style: SegmentStyle) -> Self {
        Self { start, end, style }
    }

    /// Get segment length.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Preedit segment styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SegmentStyle {
    /// Normal, uncommitted text
    #[default]
    Normal,
    /// Currently being converted (highlighted)
    Converting,
    /// Selected for conversion
    Selected,
    /// Already converted
    Converted,
}

#[cfg(test)]
mod tests {
    use super::{Preedit, PreeditSegment, SegmentStyle};

    #[test]
    fn test_preedit_new() {
        let preedit = Preedit::new();
        assert!(preedit.is_empty());
        assert_eq!(preedit.cursor(), 0);
    }

    #[test]
    fn test_preedit_with_text() {
        let preedit = Preedit::with_text("にほん");
        assert_eq!(preedit.text(), "にほん");
        assert_eq!(preedit.len(), 3);
        assert_eq!(preedit.cursor(), 3);
    }

    #[test]
    fn test_preedit_width() {
        let preedit = Preedit::with_text("にほん");
        assert_eq!(preedit.width(), 6); // 3 chars * 2 columns
    }

    #[test]
    fn test_preedit_insert() {
        let mut preedit = Preedit::new();
        preedit.insert("あ");
        assert_eq!(preedit.text(), "あ");
        assert_eq!(preedit.cursor(), 1);

        preedit.insert("い");
        assert_eq!(preedit.text(), "あい");
        assert_eq!(preedit.cursor(), 2);
    }

    #[test]
    fn test_preedit_backspace() {
        let mut preedit = Preedit::with_text("あいう");
        preedit.backspace();
        assert_eq!(preedit.text(), "あい");
        assert_eq!(preedit.cursor(), 2);
    }

    #[test]
    fn test_preedit_cursor_movement() {
        let mut preedit = Preedit::with_text("あいう");
        assert_eq!(preedit.cursor(), 3);

        preedit.move_left();
        assert_eq!(preedit.cursor(), 2);

        preedit.move_start();
        assert_eq!(preedit.cursor(), 0);

        preedit.move_end();
        assert_eq!(preedit.cursor(), 3);
    }

    #[test]
    fn test_preedit_segment() {
        let segment = PreeditSegment::new(0, 3, SegmentStyle::Converting);
        assert_eq!(segment.len(), 3);
        assert_eq!(segment.style, SegmentStyle::Converting);
    }
}
