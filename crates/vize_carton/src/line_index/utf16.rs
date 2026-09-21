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
    // UTF-16 never needs more code units than UTF-8 needs bytes. Only inspect
    // the requested prefix: column zero must stay O(1) even on a huge line,
    // and Unicode later in the line cannot affect an ASCII prefix's offset.
    let prefix = text.as_bytes().get(..column)?;
    if prefix.is_ascii() {
        return Some(column);
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

#[cfg(test)]
mod tests {
    use super::utf16_offset;

    #[test]
    fn every_utf16_boundary_matches_the_authored_characters() {
        for source in ["", "abc", "ascii😀é€", "😀ascii", "é😀x"] {
            let mut column = 0;
            for (byte, ch) in source.char_indices() {
                assert_eq!(utf16_offset(source, column), Some(byte));
                if ch.len_utf16() == 2 {
                    assert_eq!(utf16_offset(source, column + 1), None);
                }
                column += ch.len_utf16() as u32;
            }
            assert_eq!(utf16_offset(source, column), Some(source.len()));
            assert_eq!(utf16_offset(source, column + 1), None);
            assert_eq!(utf16_offset(source, u32::MAX), None);
        }
    }
}
