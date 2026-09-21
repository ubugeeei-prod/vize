//! UTF-16 column arithmetic without allocating an encoded string.

/// Count UTF-16 units. Most generated and authored code is ASCII; the standard
/// library checks that case in wide chunks before any scalar decoding.
#[inline]
pub fn utf16_len(text: &str) -> usize {
    if text.is_ascii() {
        text.len()
    } else {
        text.chars().map(char::len_utf16).sum()
    }
}

/// Keep the historical byte-position behavior: a position within a UTF-8
/// character counts that whole character, without slicing at an invalid boundary.
#[inline]
pub(super) fn prefix_len(text: &str, bytes: usize) -> usize {
    let mut end = bytes.min(text.len());
    while !text.is_char_boundary(end) {
        end += 1;
    }
    utf16_len(&text[..end])
}

/// Reject positions between surrogate halves and beyond the text.
#[inline]
pub fn utf16_offset(text: &str, column: u32) -> Option<usize> {
    let column = column as usize;
    if text.is_ascii() {
        return (column <= text.len()).then_some(column);
    }
    let mut units = 0;
    for (at, ch) in text.char_indices() {
        if units == column {
            return Some(at);
        }
        units += ch.len_utf16();
        if units > column {
            return None;
        }
    }
    (units == column).then_some(text.len())
}
