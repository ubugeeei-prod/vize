//! Server text and JavaScript string encodings retain their whole authored span.

use crate::write::{LinkSink, Writer};
use vize_l0::{Span, String};

/// HTML data/attribute escaping precedes template-string escaping. Every input
/// character is consumed exactly once, so entity-looking values never recurse.
pub(super) fn template<L: LinkSink>(writer: &mut Writer<L>, text: &str, span: Span, html: bool) {
    let mut encoded = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' if html => encoded.push_str("&amp;"),
            '<' if html => encoded.push_str("&lt;"),
            '>' if html => encoded.push_str("&gt;"),
            '"' if html => encoded.push_str("&quot;"),
            '\'' if html => encoded.push_str("&#39;"),
            '\\' => encoded.push_str("\\\\"),
            '`' => encoded.push_str("\\`"),
            '$' => encoded.push_str("\\$"),
            '\r' => encoded.push_str("\\r"),
            character => encoded.push(character),
        }
    }
    writer.push_linked(&encoded, span);
}

pub(super) fn quoted<L: LinkSink>(writer: &mut Writer<L>, text: &str, span: Span) {
    // A string contains no unsupported serializer value.
    let encoded = serde_json::to_string(text).unwrap_or_default();
    writer.push_linked(&encoded, span);
}

pub(super) fn property<L: LinkSink>(writer: &mut Writer<L>, name: &str, span: Span) {
    let mut bytes = name.bytes();
    if bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
    {
        writer.push_linked(name, span);
    } else {
        quoted(writer, name, span);
    }
}
