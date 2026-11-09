//! Published first-scalar entity API over the shared complete-value decoder.

use crate::markup::entity::{DecodedEntity, EntityContext, decode_one};
use htmlize::Context;

/// Preserve the decoded first scalar and exact authored bytes consumed.
pub fn try_decode_entity(input: &[u8], context: Context) -> Option<(char, usize)> {
    let context = match context {
        Context::General => EntityContext::Text,
        Context::Attribute => EntityContext::Attribute,
    };
    let (value, consumed) = decode_one(input, context)?;
    let first = match value {
        DecodedEntity::Named(text) => text.chars().next()?,
        DecodedEntity::Numeric(ch) => ch,
    };
    Some((first, consumed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_named_semicolon() {
        let s = b"&amp;rest";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '&');
        assert_eq!(n, 5);
        assert_eq!(&s[n..], b"rest");
    }

    #[test]
    fn general_named_no_semicolon_longest() {
        let s = b"&timesX";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '\u{00d7}');
        assert_eq!(n, 6);
    }

    #[test]
    fn attribute_times_x_not_entity() {
        assert!(try_decode_entity(b"&timesX", Context::Attribute).is_none());
    }

    #[test]
    fn numeric_dec() {
        let s = b"&#38;z";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '&');
        assert_eq!(n, 5);
    }

    /// `&` + `&amp;`: leading `&` alone is not a valid entity (next byte is `&`).
    #[test]
    fn double_ampersand_amp_from_start_is_none() {
        assert!(try_decode_entity(b"&&amp;", Context::General).is_none());
        assert!(try_decode_entity(b"&&amp;", Context::Attribute).is_none());
    }

    /// Same bytes as tokenizer would pass after consuming the first literal `&`.
    #[test]
    fn double_ampersand_decode_second_reference() {
        let s = b"&&amp;";
        let (c, n) = try_decode_entity(&s[1..], Context::General).unwrap();
        assert_eq!(c, '&');
        assert_eq!(n, 5);
        assert_eq!(1 + n, s.len());
    }

    #[test]
    fn hex_numeric() {
        let s = b"&#x26;y";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '&');
        assert_eq!(n, 6);
        assert_eq!(&s[n..], b"y");
    }

    #[test]
    fn attribute_named_with_semicolon() {
        let s = b"&lt;";
        let (c, n) = try_decode_entity(s, Context::Attribute).unwrap();
        assert_eq!(c, '<');
        assert_eq!(n, 4);
    }

    #[test]
    fn numeric_surrogate_replaced() {
        let s = b"&#55296;";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '\u{FFFD}');
        assert_eq!(n, 8);
    }

    #[test]
    fn numeric_windows_1252_mapping() {
        let s = b"&#128;";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '\u{20AC}');
        assert_eq!(n, 6);
    }

    #[test]
    fn bare_ampersand_is_none() {
        assert!(try_decode_entity(b"&", Context::General).is_none());
        assert!(try_decode_entity(b"&@", Context::General).is_none());
    }
}
