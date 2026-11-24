//! Kani checks the shipped JavaScript line-terminator step (`line_endings.rs`)
//! that the emitter's source-map line table and generated-side scan advance
//! by. ECMAScript line terminators are LF, CR, LS (U+2028) and PS (U+2029),
//! and CR LF is one terminator sequence. The spec decodes the scalar at the
//! offset with `char_indices`, not the byte patterns the kernel matches.
//!
//! Bound: every UTF-8 text of at most four bytes; the offset ranges over the
//! whole `usize` domain. The kernel has no loop, so the check covers every
//! terminator and every scalar a four-byte window can hold.

mod line_endings;

#[cfg(kani)]
mod proofs {
    use super::line_endings::line_break_len;

    #[kani::proof]
    #[kani::unwind(6)]
    #[kani::solver(cadical)]
    fn line_break_len_is_the_width_of_the_terminator_at_the_offset() {
        let bytes: [u8; 4] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= bytes.len());
        let Some(text) = bytes
            .get(..len)
            .and_then(|bytes| core::str::from_utf8(bytes).ok())
        else {
            return;
        };
        let offset: usize = kani::any();
        let mut scalars = text.get(offset..).unwrap_or_default().chars();
        let expected = match (scalars.next(), scalars.next()) {
            // An offset inside a scalar or past the end starts no terminator.
            _ if !text.is_char_boundary(offset) => 0,
            (Some('\r'), Some('\n')) => 2,
            (Some(ch @ ('\n' | '\r' | '\u{2028}' | '\u{2029}')), _) => ch.len_utf8(),
            _ => 0,
        };
        assert_eq!(line_break_len(text, offset), expected);
        kani::cover!(expected == 2, "CR LF");
        kani::cover!(expected == 3, "a line or paragraph separator");
        kani::cover!(
            expected == 1 && text.as_bytes().get(offset) == Some(&b'\r'),
            "a lone CR"
        );
        kani::cover!(
            expected == 0 && offset < text.len() && !text.is_char_boundary(offset),
            "an offset inside a multi-byte scalar"
        );
    }
}
