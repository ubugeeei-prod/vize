//! Kani checks the shipped UTF-16 column kernel (`utf16.rs`) that every LSP
//! position and diagnostic column goes through, against the definitions of
//! the two encodings: a UTF-8 scalar starts at every byte that is not a
//! continuation byte (`10xxxxxx`), and it takes two UTF-16 code units exactly
//! when its UTF-8 form is four bytes long (a leading byte `11110xxx`), one
//! otherwise. The spec functions below read only those facts off the bytes.
//!
//! Bound: every UTF-8 string of at most four bytes (an astral scalar, a
//! multi-byte scalar and ASCII, or four ASCII bytes). Columns and byte offsets
//! range over their whole `u32` and `usize` domains.

mod utf16;

#[cfg(kani)]
mod proofs {
    use super::utf16::{prefix_len, utf16_len, utf16_offset};

    const MAX_BYTES: usize = 4;

    fn with_text(max: usize, check: impl FnOnce(&str)) {
        let bytes: [u8; MAX_BYTES] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= max);
        if let Some(text) = bytes
            .get(..len)
            .and_then(|bytes| core::str::from_utf8(bytes).ok())
        {
            check(text);
        }
    }

    /// For each byte offset `0..=len`: whether a scalar starts there, and the
    /// UTF-16 units of the scalars that start before it. A continuation byte
    /// adds no unit, so an offset inside a scalar already counts it whole.
    struct Units {
        start: [bool; MAX_BYTES + 1],
        before: [usize; MAX_BYTES + 1],
    }

    fn units(text: &str) -> Units {
        let mut units = Units {
            start: [true; MAX_BYTES + 1],
            before: [0; MAX_BYTES + 1],
        };
        let mut total = 0;
        for (at, &byte) in text.as_bytes().iter().enumerate() {
            units.before[at] = total;
            units.start[at] = byte & 0xc0 != 0x80;
            if units.start[at] {
                total += if byte >= 0xf0 { 2 } else { 1 };
            }
        }
        units.before[text.len()] = total;
        units
    }

    #[kani::proof]
    #[kani::unwind(6)]
    fn utf16_offset_is_the_boundary_with_that_many_units() {
        with_text(MAX_BYTES, |text| {
            let spec = units(text);
            let column: u32 = kani::any();
            let mut expected = None;
            for at in 0..=text.len() {
                if spec.start[at] && spec.before[at] == column as usize {
                    expected = Some(at);
                }
            }
            let actual = utf16_offset(text, column);
            assert_eq!(actual, expected);
            kani::cover!(
                actual.is_some() && !text.is_ascii() && column > 0,
                "a column after non-ASCII text"
            );
            kani::cover!(
                actual.is_none() && (column as usize) < spec.before[text.len()],
                "a column between the halves of a surrogate pair"
            );
            kani::cover!(column == u32::MAX, "a column past every line");
        });
    }

    #[kani::proof]
    #[kani::unwind(6)]
    #[kani::solver(cadical)]
    fn utf16_len_counts_the_units_of_every_scalar() {
        with_text(MAX_BYTES, |text| {
            let counted = utf16_len(text);
            assert_eq!(counted, units(text).before[text.len()]);
            kani::cover!(counted == 2 && text.len() == 4, "an astral scalar");
            kani::cover!(counted == 4 && text.is_ascii(), "four ASCII bytes");
        });
    }

    #[kani::proof]
    #[kani::unwind(6)]
    #[kani::solver(cadical)]
    fn utf16_prefix_counts_the_units_of_whole_scalars() {
        with_text(MAX_BYTES, |text| {
            let spec = units(text);
            let offset: usize = kani::any();
            let before = prefix_len(text, offset);
            // A mid-scalar offset counts the whole scalar it falls in, so
            // columns never decrease as the offset moves forward.
            assert_eq!(before, spec.before[offset.min(text.len())]);
            kani::cover!(
                offset < text.len() && !spec.start[offset] && before > 0,
                "an offset inside a multi-byte scalar"
            );
            kani::cover!(
                offset > text.len() && before == spec.before[text.len()] && !text.is_empty(),
                "an offset past the end"
            );
        });
    }
}
