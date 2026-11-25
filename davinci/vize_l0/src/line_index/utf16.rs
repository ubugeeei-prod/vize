//! UTF-16 column arithmetic without allocating an encoded string. The byte
//! kernels in `code_units.rs` count the units; a `&str` is always UTF-8,
//! which is all their contracts ask for.

use super::code_units::{offset_of_units, units_between};

/// Count UTF-16 units. Most generated and authored code is ASCII; the kernel
/// checks that case with one vectorized pass before counting lead bytes.
#[inline]
pub fn utf16_len(text: &str) -> usize {
    units_between(text.as_bytes(), 0, text.len())
}

/// Keep the historical byte-position behavior: a position within a UTF-8
/// character counts that whole character, without slicing at an invalid boundary.
#[inline]
pub(super) fn prefix_len(text: &str, bytes: usize) -> usize {
    units_between(text.as_bytes(), 0, bytes.min(text.len()))
}

/// Reject positions between surrogate halves and beyond the text.
#[inline]
pub fn utf16_offset(text: &str, column: u32) -> Option<usize> {
    offset_of_units(text.as_bytes(), 0, text.len(), column as usize)
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
