use alloc::borrow::Cow;
use memchr::memchr;

use super::AttrSink;

pub(super) fn parse_attrs<'a>(
    bytes: &'a [u8],
    source: &'a str,
    pos: &mut usize,
    attrs: &mut impl AttrSink<'a>,
) {
    let mut pos = *pos;
    let len = bytes.len();
    // Parse attributes with zero-copy

    while bytes.get(pos).is_some_and(|&b| b != b'>') {
        // Skip whitespace
        while bytes.get(pos).is_some_and(|&b| is_whitespace_fast(b)) {
            pos += 1;
        }

        if bytes.get(pos).is_none_or(|&b| b == b'>' || b == b'/') {
            break;
        }

        // Parse attribute name
        let attr_start = pos;
        while let Some(&c) = bytes.get(pos) {
            if c == b'='
                || c == b' '
                || c == b'>'
                || c == b'/'
                || c == b'\t'
                || c == b'\n'
                || c == b'\r'
            {
                break;
            }
            pos += 1;
        }

        if pos == attr_start {
            pos += 1;
            continue;
        }

        // Zero-copy: borrow from source
        let attr_name: Cow<'a, str> =
            Cow::Borrowed(source.get(attr_start..pos).unwrap_or_default());

        // Skip whitespace
        while matches!(bytes.get(pos), Some(b' ' | b'\t')) {
            pos += 1;
        }

        let attr_value: Cow<'a, str> = if bytes.get(pos) == Some(&b'=') {
            pos += 1;

            // Skip whitespace
            while matches!(bytes.get(pos), Some(b' ' | b'\t')) {
                pos += 1;
            }

            if let Some(&quote_char) = bytes.get(pos).filter(|&&b| b == b'"' || b == b'\'') {
                pos += 1;
                let value_start = pos;

                // Use memchr for fast quote finding
                if let Some(quote_pos) = memchr(quote_char, bytes.get(pos..).unwrap_or_default()) {
                    pos += quote_pos;
                    let value = Cow::Borrowed(source.get(value_start..pos).unwrap_or_default());
                    pos += 1; // Skip closing quote
                    value
                } else {
                    // No closing quote found
                    while bytes.get(pos).is_some_and(|&b| b != quote_char) {
                        pos += 1;
                    }
                    let value = Cow::Borrowed(source.get(value_start..pos).unwrap_or_default());
                    if pos < len {
                        pos += 1;
                    }
                    value
                }
            } else {
                // Unquoted value
                let value_start = pos;
                while let Some(&c) = bytes.get(pos) {
                    if c == b' ' || c == b'>' || c == b'/' || c == b'\t' || c == b'\n' {
                        break;
                    }
                    pos += 1;
                }
                Cow::Borrowed(source.get(value_start..pos).unwrap_or_default())
            }
        } else {
            // Boolean attribute
            Cow::Borrowed("")
        };

        if !attr_name.is_empty() {
            attrs.attr(attr_name, attr_value, (attr_start, pos));
        }
    }

    *pos = pos;
}
