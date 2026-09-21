//! Terminal display of authored text: what each character prints as, and how
//! many columns it takes.
//!
//! Every column the renderer computes — where a `^` goes, how far a hanging
//! label's `|` is indented — is a sum of [`glyph`] widths over the same
//! normalized characters the source row prints, so an underline lines up with
//! its text whatever the text contains:
//!
//! - east-asian wide and fullwidth characters (CJK, kana, most emoji) take two
//!   columns ([`unicode_width`], non-CJK context: ambiguous-width characters
//!   are narrow, as every mainstream terminal defaults to);
//! - a tab prints as four spaces, as rustc does, so alignment does not depend
//!   on the terminal's tab stops;
//! - a C0 control or DEL prints as its Unicode control picture (`␀`, `␛`,
//!   `␡`, …) — a raw `\r` or escape byte in a source row would move the
//!   cursor or restyle the terminal and corrupt everything after it;
//! - bidirectional overrides and isolates print as `�` so the bytes on disk and
//!   the text on screen agree (the "Trojan Source" class), and a zero-width
//!   joiner prints as nothing so a joined cluster cannot desynchronise the
//!   width sum.

use unicode_width::UnicodeWidthChar;

/// What one authored character prints as, and its width in columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Glyph {
    /// The character itself.
    Char(char, usize),
    /// A fixed replacement.
    Replacement(&'static str, usize),
}

impl Glyph {
    /// Columns this glyph occupies.
    pub(crate) const fn width(self) -> usize {
        match self {
            Self::Char(_, width) | Self::Replacement(_, width) => width,
        }
    }
}

/// Four spaces: a tab's display.
const TAB: &str = "    ";

/// The display of `ch`.
pub(crate) fn glyph(ch: char) -> Glyph {
    match ch {
        '\t' => Glyph::Replacement(TAB, TAB.len()),
        '\u{0}'..='\u{8}' | '\u{a}'..='\u{1f}' => {
            // U+2400 CONTROL PICTURES mirror C0 one to one.
            let picture = char::from_u32(0x2400 + ch as u32).unwrap_or('\u{fffd}');
            Glyph::Char(picture, 1)
        }
        '\u{7f}' => Glyph::Char('\u{2421}', 1),
        '\u{200d}' => Glyph::Replacement("", 0),
        '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' => Glyph::Char('\u{fffd}', 1),
        _ => Glyph::Char(ch, ch.width().unwrap_or(0)),
    }
}

/// Display width of `text` after normalization.
pub(crate) fn width(text: &str) -> usize {
    text.chars().map(|ch| glyph(ch).width()).sum()
}

/// Append the display of `text` to `out`.
pub(crate) fn push_display(out: &mut vize_s0::String, text: &str) {
    for ch in text.chars() {
        match glyph(ch) {
            Glyph::Char(shown, _) => out.push(shown),
            Glyph::Replacement(shown, _) => out.push_str(shown),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Glyph, glyph, push_display, width};
    use vize_s0::String;

    #[test]
    fn narrow_wide_and_zero_width_characters_measure_by_terminal_columns() {
        assert_eq!(width("abc"), 3);
        assert_eq!(width("名前"), 4);
        assert_eq!(width("ｱ"), 1);
        assert_eq!(width("Ａ"), 2);
        assert_eq!(width("e\u{301}"), 1);
        assert_eq!(width("🦀"), 2);
    }

    #[test]
    fn tabs_controls_and_bidi_marks_print_as_fixed_width_stand_ins() {
        assert_eq!(glyph('\t'), Glyph::Replacement("    ", 4));
        assert_eq!(glyph('\u{0}'), Glyph::Char('\u{2400}', 1));
        assert_eq!(glyph('\r'), Glyph::Char('\u{240d}', 1));
        assert_eq!(glyph('\u{1b}'), Glyph::Char('\u{241b}', 1));
        assert_eq!(glyph('\u{7f}'), Glyph::Char('\u{2421}', 1));
        assert_eq!(glyph('\u{202e}'), Glyph::Char('\u{fffd}', 1));
        assert_eq!(glyph('\u{2067}'), Glyph::Char('\u{fffd}', 1));
        assert_eq!(glyph('\u{200d}'), Glyph::Replacement("", 0));

        let mut shown = String::new("");
        push_display(&mut shown, "a\tb\u{1b}[31mc\u{202e}d");
        assert_eq!(shown.as_str(), "a    b\u{241b}[31mc\u{fffd}d");
        assert_eq!(width("a\tb\u{1b}[31mc\u{202e}d"), 14);
    }
}
