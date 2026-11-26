//! JavaScript line terminators, with CRLF counted as one boundary.
//!
//! ECMAScript's line terminators are LF, CR, LS (U+2028) and PS (U+2029), and
//! CR LF is one terminator sequence. The kernel reads their UTF-8 forms
//! (`0A`, `0D`, `E2 80 A8`, `E2 80 A9`) off the bytes. Creusot proves the
//! width against the definition in `creusot.rs` beside this file.

#[cfg(creusot)]
use crate::terminator_width;
#[cfg(creusot)]
use creusot_std::prelude::*;

/// The width of the line terminator sequence that starts at `offset`, or 0
/// when none starts there: inside a scalar, past the end, or at any other
/// byte.
#[cfg_attr(creusot, ensures(result@ == terminator_width(bytes@, offset@)))]
#[inline]
pub(crate) fn line_break_len(bytes: &[u8], offset: usize) -> usize {
    match bytes.get(offset) {
        Some(&b'\r') => {
            if matches!(bytes.get(offset + 1), Some(&b'\n')) {
                2
            } else {
                1
            }
        }
        Some(&b'\n') => 1,
        Some(&0xe2)
            if matches!(bytes.get(offset + 1), Some(&0x80))
                && matches!(bytes.get(offset + 2), Some(&(0xa8 | 0xa9))) =>
        {
            3
        }
        _ => 0,
    }
}
