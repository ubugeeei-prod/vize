//! Text width calculation with CJK support.

use compact_str::{CompactString, ToCompactString};
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

/// Calculate display width of text in terminal columns.
pub struct TextWidth;

impl TextWidth {
    /// Calculate the display width of a string.
    ///
    /// This accounts for:
    /// - East Asian Wide characters (CJK) taking 2 columns
    /// - Control characters taking 0 columns
    /// - Normal ASCII taking 1 column
    #[inline]
    pub fn width(s: &str) -> usize {
        UnicodeWidthStr::width(s)
    }

    /// Calculate the display width of a character.
    #[inline]
    pub fn char_width(c: char) -> usize {
        UnicodeWidthChar::width(c).unwrap_or(0)
    }

    /// Check if a character is wide (takes 2 columns).
    #[inline]
    pub fn is_wide(c: char) -> bool {
        UnicodeWidthChar::width(c).unwrap_or(0) > 1
    }

    /// Check if a character is zero-width.
    #[inline]
    pub fn is_zero_width(c: char) -> bool {
        UnicodeWidthChar::width(c).unwrap_or(0) == 0
    }

    /// Truncate string to fit within max_width columns.
    /// Returns (truncated_str, actual_width).
    pub fn truncate(s: &str, max_width: usize) -> (&str, usize) {
        let mut width = 0;
        let mut end = 0;

        for (i, c) in s.char_indices() {
            let char_width = Self::char_width(c);
            if width + char_width > max_width {
                break;
            }
            width += char_width;
            end = i + c.len_utf8();
        }

        (&s[..end], width)
    }

    /// Truncate string with ellipsis if needed.
    /// The ellipsis is "..." (3 columns).
    #[allow(clippy::disallowed_macros)]
    pub fn truncate_with_ellipsis(s: &str, max_width: usize) -> CompactString {
        let width = Self::width(s);
        if width <= max_width {
            return s.to_compact_string();
        }

        if max_width < 3 {
            return std::iter::repeat_n('.', max_width).collect();
        }

        let target_width = max_width - 3;
        let (truncated, _) = Self::truncate(s, target_width);
        CompactString::from(format!("{}...", truncated))
    }

    /// Pad string to specified width.
    #[allow(clippy::disallowed_macros)]
    pub fn pad_right(s: &str, target_width: usize) -> CompactString {
        let current_width = Self::width(s);
        if current_width >= target_width {
            return s.to_compact_string();
        }

        let padding = target_width - current_width;
        CompactString::from(format!("{}{}", s, " ".repeat(padding)))
    }

    /// Pad string to specified width (left padding).
    #[allow(clippy::disallowed_macros)]
    pub fn pad_left(s: &str, target_width: usize) -> CompactString {
        let current_width = Self::width(s);
        if current_width >= target_width {
            return s.to_compact_string();
        }

        let padding = target_width - current_width;
        CompactString::from(format!("{}{}", " ".repeat(padding), s))
    }

    /// Center string within specified width.
    #[allow(clippy::disallowed_macros)]
    pub fn center(s: &str, target_width: usize) -> CompactString {
        let current_width = Self::width(s);
        if current_width >= target_width {
            return s.to_compact_string();
        }

        let total_padding = target_width - current_width;
        let left_padding = total_padding / 2;
        let right_padding = total_padding - left_padding;

        CompactString::from(format!(
            "{}{}{}",
            " ".repeat(left_padding),
            s,
            " ".repeat(right_padding)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::TextWidth;

    #[test]
    fn test_ascii_width() {
        assert_eq!(TextWidth::width("Hello"), 5);
        assert_eq!(TextWidth::width(""), 0);
    }

    #[test]
    fn test_cjk_width() {
        // Japanese characters are 2 columns wide
        assert_eq!(TextWidth::width("あ"), 2);
        assert_eq!(TextWidth::width("あいう"), 6);
        assert_eq!(TextWidth::width("Hello世界"), 9); // 5 + 2*2
    }

    #[test]
    fn test_emoji_width() {
        // Most emojis are 2 columns wide
        assert_eq!(TextWidth::char_width('😀'), 2);
    }

    #[test]
    fn test_char_width() {
        assert_eq!(TextWidth::char_width('A'), 1);
        assert_eq!(TextWidth::char_width('あ'), 2);
        assert_eq!(TextWidth::char_width('\0'), 0);
    }

    #[test]
    fn test_is_wide() {
        assert!(!TextWidth::is_wide('A'));
        assert!(TextWidth::is_wide('あ'));
        assert!(TextWidth::is_wide('中'));
    }

    #[test]
    fn test_truncate() {
        let (s, w) = TextWidth::truncate("Hello", 3);
        assert_eq!(s, "Hel");
        assert_eq!(w, 3);

        let (s, w) = TextWidth::truncate("あいう", 4);
        assert_eq!(s, "あい");
        assert_eq!(w, 4);

        // Can't fit a wide char in remaining 1 column
        let (s, w) = TextWidth::truncate("あいう", 5);
        assert_eq!(s, "あい");
        assert_eq!(w, 4);
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        assert_eq!(TextWidth::truncate_with_ellipsis("Hello", 10), "Hello");
        assert_eq!(
            TextWidth::truncate_with_ellipsis("Hello World", 8),
            "Hello..."
        );
    }

    #[test]
    fn test_pad_right() {
        assert_eq!(TextWidth::pad_right("Hi", 5), "Hi   ");
        assert_eq!(TextWidth::pad_right("あ", 5), "あ   "); // 2 + 3
    }

    #[test]
    fn test_pad_left() {
        assert_eq!(TextWidth::pad_left("Hi", 5), "   Hi");
    }

    #[test]
    fn test_center() {
        assert_eq!(TextWidth::center("Hi", 6), "  Hi  ");
        assert_eq!(TextWidth::center("Hi", 7), "  Hi   ");
    }
}
