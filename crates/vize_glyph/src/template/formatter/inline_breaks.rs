//! Authored line breaks between inline text nodes under the Prettier preset.
//!
//! Prettier joins the words of one text node, but keeps a line break between a
//! text node and an interpolation, or between two interpolations, when either
//! side has a line break both before and after it, or a blank line separates
//! them.

use super::parse_interpolation_range;
use crate::template::helpers::is_whitespace;

/// The last inline node of the current text run.
#[derive(Clone, Copy)]
pub(super) enum InlineNode {
    None,
    Text { leading_break: bool },
    Interpolation { leading_break: bool },
}

impl InlineNode {
    /// Advance over `source[start..end]`, text the run just buffered.
    pub(super) fn track(self, source: &[u8], start: usize, end: usize) -> Self {
        let mut node = self;
        let mut cursor = start;
        while cursor < end {
            if source.get(cursor).copied().is_some_and(is_whitespace) {
                cursor += 1;
                continue;
            }
            if let Some(close) = interpolation_end(source, cursor) {
                node = Self::Interpolation {
                    leading_break: line_break_before(source, cursor),
                };
                cursor = close;
                continue;
            }
            if !matches!(node, Self::Text { .. }) {
                node = Self::Text {
                    leading_break: line_break_before(source, cursor),
                };
            }
            while cursor < end
                && !source.get(cursor).copied().is_some_and(is_whitespace)
                && interpolation_end(source, cursor).is_none()
            {
                cursor += 1;
            }
        }
        node
    }

    /// The next node start when the authored line break at `newline` stays,
    /// and whether an authored blank line stays with it.
    pub(super) fn kept_line_break(self, source: &[u8], newline: usize) -> Option<(usize, bool)> {
        let (text, leading_break) = match self {
            Self::None => return None,
            Self::Text { leading_break } => (true, leading_break),
            Self::Interpolation { leading_break } => (false, leading_break),
        };
        let mut next = newline;
        let mut line_breaks = 0;
        while let Some(&byte) = source.get(next).filter(|byte| is_whitespace(**byte)) {
            next += 1;
            // CRLF is one line break.
            if byte == b'\n' || (byte == b'\r' && source.get(next) != Some(&b'\n')) {
                line_breaks += 1;
            }
        }
        let trailing_break = if let Some(close) = interpolation_end(source, next) {
            line_break_after(source, close)
        } else if text || matches!(source.get(next), None | Some(b'<')) {
            // Words of one text node fill; markup keeps its own layout.
            return None;
        } else {
            line_break_after(source, text_node_end(source, next))
        };
        let blank_line = line_breaks >= 2;
        (blank_line || leading_break || trailing_break).then_some((next, blank_line))
    }
}

fn interpolation_end(source: &[u8], at: usize) -> Option<usize> {
    if source.get(at..at + 2) != Some(b"{{") {
        return None;
    }
    parse_interpolation_range(source, at).map(|(_, _, close)| close)
}

/// End of the last word of the text node starting at `start`.
fn text_node_end(source: &[u8], start: usize) -> usize {
    let mut cursor = start;
    let mut end = start;
    while let Some(&byte) = source.get(cursor) {
        if byte == b'<' || interpolation_end(source, cursor).is_some() {
            break;
        }
        cursor += 1;
        if !is_whitespace(byte) {
            end = cursor;
        }
    }
    end
}

fn line_break_before(source: &[u8], at: usize) -> bool {
    source
        .get(..at)
        .unwrap_or_default()
        .iter()
        .rev()
        .find(|byte| !matches!(byte, b' ' | b'\t'))
        .is_some_and(|byte| matches!(byte, b'\n' | b'\r'))
}

fn line_break_after(source: &[u8], at: usize) -> bool {
    source
        .get(at..)
        .unwrap_or_default()
        .iter()
        .find(|byte| !matches!(byte, b' ' | b'\t'))
        .is_some_and(|byte| matches!(byte, b'\n' | b'\r'))
}
