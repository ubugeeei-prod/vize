//! Entity-decoding hook.
//!
//! Decoding runs only where an `&` is present: the lexer calls
//! [`decode_one`] at an `&`, and embed construction asks
//! [`needs_decoding`] before building a decode map, so entity-free input
//! (almost all of it) pays one byte scan and nothing else.

#![expect(clippy::todo, reason = "skeleton: #6835")]

/// Where a character reference appears; attribute values follow the WHATWG
/// legacy rule for unterminated named references.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityContext {
    Text,
    Attribute,
}

/// Whether `bytes` contains any character reference candidate.
#[inline]
pub fn needs_decoding(bytes: &[u8]) -> bool {
    bytes.contains(&b'&')
}

/// Decode at most one character reference at the start of `input`.
///
/// Returns the first decoded scalar and the number of bytes consumed
/// (`&`, name or number, optional `;`), or `None` when `input` does not
/// start with a valid reference and the `&` is literal text.
///
/// # Panics
///
/// Always, until the generic entity hook replaces the moved compatibility
/// decoder after parity validation (#6835).
pub fn decode_one(_input: &[u8], _context: EntityContext) -> Option<(char, usize)> {
    todo!("#6835: move entity decoding into vize_l1::markup::entity")
}

#[cfg(test)]
mod tests {
    use super::needs_decoding;

    #[test]
    fn only_ampersands_need_decoding() {
        assert!(!needs_decoding(b"a || b"));
        assert!(needs_decoding(b"a &amp;&amp; b"));
    }
}
