//! Text wrapping utilities.

use super::segment::segment;
use super::width::TextWidth;
use compact_str::CompactString;

/// Text wrapping mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WrapMode {
    /// No wrapping - text may overflow
    #[default]
    NoWrap,
    /// Wrap at word boundaries
    Word,
    /// Wrap at grapheme boundaries
    Char,
    /// Truncate at the end with ellipsis
    Truncate,
    /// Truncate at the end with ellipsis
    TruncateEnd,
    /// Truncate at the start with ellipsis
    TruncateStart,
    /// Truncate in the middle with ellipsis
    TruncateMiddle,
}

/// Text wrapper for terminal output.
pub struct TextWrap;

impl TextWrap {
    /// Wrap text to fit within max_width columns.
    pub fn wrap(text: &str, max_width: usize, mode: WrapMode) -> Vec<CompactString> {
        match mode {
            WrapMode::NoWrap => vec![CompactString::from(text)],
            WrapMode::Word => Self::wrap_word(text, max_width),
            WrapMode::Char => Self::wrap_char(text, max_width),
            WrapMode::Truncate | WrapMode::TruncateEnd => vec![Self::truncate_end(text, max_width)],
            WrapMode::TruncateStart => vec![Self::truncate_start(text, max_width)],
            WrapMode::TruncateMiddle => vec![Self::truncate_middle(text, max_width)],
        }
    }

    fn ellipsis(max_width: usize) -> CompactString {
        std::iter::repeat_n('.', max_width.min(3)).collect()
    }

    fn truncate_end(text: &str, max_width: usize) -> CompactString {
        if TextWidth::width(text) <= max_width {
            return CompactString::from(text);
        }

        if max_width <= 3 {
            return Self::ellipsis(max_width);
        }

        let target_width = max_width - 3;
        let mut result = CompactString::default();
        let mut width = 0;

        for seg in segment(text) {
            if width + seg.width > target_width {
                break;
            }

            width += seg.width;
            result.push_str(&seg.grapheme);
        }

        result.push_str("...");
        result
    }

    fn truncate_start(text: &str, max_width: usize) -> CompactString {
        if TextWidth::width(text) <= max_width {
            return CompactString::from(text);
        }

        if max_width <= 3 {
            return Self::ellipsis(max_width);
        }

        let target_width = max_width - 3;
        let mut suffix = Vec::new();
        let mut width = 0;

        for seg in segment(text).collect::<Vec<_>>().into_iter().rev() {
            if width + seg.width > target_width {
                break;
            }

            width += seg.width;
            suffix.push(seg.grapheme);
        }

        let mut result = CompactString::from("...");
        for item in suffix.into_iter().rev() {
            result.push_str(&item);
        }
        result
    }

    fn truncate_middle(text: &str, max_width: usize) -> CompactString {
        if TextWidth::width(text) <= max_width {
            return CompactString::from(text);
        }

        if max_width <= 3 {
            return Self::ellipsis(max_width);
        }

        let target_width = max_width - 3;
        let prefix_target = target_width.div_ceil(2);
        let suffix_target = target_width / 2;

        let mut prefix = CompactString::default();
        let mut prefix_width = 0;
        let segments = segment(text).collect::<Vec<_>>();

        for seg in &segments {
            if prefix_width + seg.width > prefix_target {
                break;
            }

            prefix_width += seg.width;
            prefix.push_str(&seg.grapheme);
        }

        let mut suffix = Vec::new();
        let mut suffix_width = 0;
        for seg in segments.into_iter().rev() {
            if suffix_width + seg.width > suffix_target {
                break;
            }

            suffix_width += seg.width;
            suffix.push(seg.grapheme);
        }

        prefix.push_str("...");
        for item in suffix.into_iter().rev() {
            prefix.push_str(&item);
        }
        prefix
    }

    /// Wrap at word boundaries.
    fn wrap_word(text: &str, max_width: usize) -> Vec<CompactString> {
        if max_width == 0 {
            return vec![];
        }

        let mut lines = Vec::new();
        let mut current_line = CompactString::default();
        let mut current_width = 0;

        for word in text.split_inclusive(|c: char| c.is_whitespace()) {
            let word_width = TextWidth::width(word);
            let trimmed = word.trim_end();
            let trimmed_width = TextWidth::width(trimmed);

            // If word doesn't fit on current line
            if current_width + trimmed_width > max_width {
                // If current line has content, finish it
                if current_width > 0 {
                    lines.push(CompactString::from(current_line.trim_end()));
                    current_line.clear();
                    current_width = 0;
                }

                // If word is longer than max_width, use char wrap
                if trimmed_width > max_width {
                    let sub_lines = Self::wrap_char(trimmed, max_width);
                    let sub_len = sub_lines.len();
                    for (i, sub_line) in sub_lines.into_iter().enumerate() {
                        if i < sub_len - 1 {
                            lines.push(sub_line);
                        } else {
                            current_line.push_str(&sub_line);
                            current_width = TextWidth::width(&sub_line);
                        }
                    }
                    continue;
                }
            }

            current_line.push_str(word);
            current_width += word_width;
        }

        // Don't forget the last line
        if !current_line.is_empty() {
            lines.push(CompactString::from(current_line.trim_end()));
        }

        if lines.is_empty() {
            lines.push(CompactString::new(""));
        }

        lines
    }

    /// Wrap at character boundaries.
    fn wrap_char(text: &str, max_width: usize) -> Vec<CompactString> {
        if max_width == 0 {
            return vec![];
        }

        let mut lines = Vec::new();
        let mut current_line = CompactString::default();
        let mut current_width = 0;

        for seg in segment(text) {
            // If adding this segment would exceed max width
            if current_width + seg.width > max_width {
                // Finish current line
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                }
                current_line.clear();
                current_width = 0;

                // Handle wide char at start of line that's wider than max
                if seg.width > max_width {
                    // Can't fit this char, skip it or use placeholder
                    lines.push(CompactString::from("?"));
                    continue;
                }
            }

            current_line.push_str(&seg.grapheme);
            current_width += seg.width;
        }

        // Don't forget the last line
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(CompactString::new(""));
        }

        lines
    }

    /// Split text into lines (preserving existing newlines).
    pub fn split_lines(text: &str) -> Vec<&str> {
        text.lines().collect()
    }

    /// Wrap text and return with line count.
    pub fn wrap_with_info(
        text: &str,
        max_width: usize,
        mode: WrapMode,
    ) -> (Vec<CompactString>, usize) {
        let lines = Self::wrap(text, max_width, mode);
        let count = lines.len();
        (lines, count)
    }

    /// Calculate how many lines text would take.
    pub fn line_count(text: &str, max_width: usize, mode: WrapMode) -> usize {
        Self::wrap(text, max_width, mode).len()
    }
}

/// A wrapped line with metadata.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WrappedLine {
    /// The text content
    pub content: CompactString,
    /// Display width
    pub width: usize,
    /// Whether this line was wrapped from the previous
    pub is_continuation: bool,
}

#[allow(dead_code)]
impl WrappedLine {
    /// Create a new wrapped line.
    pub fn new(content: impl Into<CompactString>, is_continuation: bool) -> Self {
        let content: CompactString = content.into();
        let width = TextWidth::width(&content);
        Self {
            content,
            width,
            is_continuation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TextWrap, WrapMode};

    #[test]
    fn test_wrap_no_wrap() {
        let lines = TextWrap::wrap("Hello World", 5, WrapMode::NoWrap);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].as_str(), "Hello World");
    }

    #[test]
    fn test_wrap_word() {
        let lines = TextWrap::wrap("Hello World", 6, WrapMode::Word);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].as_str(), "Hello");
        assert_eq!(lines[1].as_str(), "World");
    }

    #[test]
    fn test_wrap_word_long() {
        let lines = TextWrap::wrap("Supercalifragilistic", 5, WrapMode::Word);
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_wrap_char() {
        let lines = TextWrap::wrap("Hello", 3, WrapMode::Char);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].as_str(), "Hel");
        assert_eq!(lines[1].as_str(), "lo");
    }

    #[test]
    fn test_wrap_char_cjk() {
        let lines = TextWrap::wrap("あいうえお", 4, WrapMode::Char);
        // Each CJK char is 2 wide, so 2 chars per line
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].as_str(), "あい");
        assert_eq!(lines[1].as_str(), "うえ");
        assert_eq!(lines[2].as_str(), "お");
    }

    #[test]
    fn test_wrap_truncate() {
        let lines = TextWrap::wrap("Hello World", 8, WrapMode::Truncate);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].as_str(), "Hello...");
    }

    #[test]
    fn test_wrap_truncate_start() {
        let lines = TextWrap::wrap("Hello World", 8, WrapMode::TruncateStart);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].as_str(), "...World");
    }

    #[test]
    fn test_wrap_truncate_middle() {
        let lines = TextWrap::wrap("Hello World", 9, WrapMode::TruncateMiddle);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].as_str(), "Hel...rld");
    }

    #[test]
    fn test_wrap_truncate_cjk_width() {
        let lines = TextWrap::wrap("あいうえお", 7, WrapMode::TruncateEnd);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].as_str(), "あい...");
    }

    #[test]
    fn test_split_lines() {
        let lines = TextWrap::split_lines("Hello\nWorld");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Hello");
        assert_eq!(lines[1], "World");
    }

    #[test]
    fn test_line_count() {
        let count = TextWrap::line_count("Hello World", 6, WrapMode::Word);
        assert_eq!(count, 2);
    }
}
