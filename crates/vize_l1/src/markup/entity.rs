//! Entity-decoding hook.
//!
//! Decoding runs only where an `&` is present: the lexer calls
//! [`decode_one`] at an `&`, and embed construction asks
//! [`needs_decoding`] before building a decode map, so entity-free input
//! (almost all of it) pays one byte scan and nothing else.

pub(crate) mod decode;

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
/// Rules follow WHATWG (`htmlize`'s entity table).
#[inline]
pub fn decode_one(input: &[u8], context: EntityContext) -> Option<(char, usize)> {
    let context = match context {
        EntityContext::Text => htmlize::Context::General,
        EntityContext::Attribute => htmlize::Context::Attribute,
    };
    decode::try_decode_entity(input, context)
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
