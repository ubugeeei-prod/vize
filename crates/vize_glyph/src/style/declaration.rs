use super::comment_scan::consume_css_escape;
use vize_l0::String;

/// Recognize only a property name followed by its actual colon. Selectors,
/// at-rules and punctuation inside CSS escapes cannot become declarations.
fn colon(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let first = *bytes.first()?;
    if !(first.is_ascii_alphabetic() || matches!(first, b'-' | b'_' | b'\\') || first >= 0x80) {
        return None;
    }
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        if let Some(next) = consume_css_escape(bytes, index) {
            index = next;
        } else if byte == b':' {
            return Some(index);
        } else if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') || byte >= 0x80 {
            index += 1;
        } else if byte.is_ascii_whitespace() {
            while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
                index += 1;
            }
            return (bytes.get(index) == Some(&b':')).then_some(index);
        } else {
            return None;
        }
    }
    None
}

pub(super) fn starts_custom_value(source: &str) -> bool {
    if colon(source).is_none() {
        return false;
    }
    let bytes = source.as_bytes();
    leading_dash(bytes, 0)
        .and_then(|next| leading_dash(bytes, next))
        .is_some()
}

fn leading_dash(bytes: &[u8], from: usize) -> Option<usize> {
    if bytes.get(from) == Some(&b'-') {
        return Some(from + 1);
    }
    let next = consume_css_escape(bytes, from)?;
    if bytes.get(from + 1) == Some(&b'-') {
        return Some(next);
    }
    let mut value = 0;
    let mut digits = 0;
    for &byte in bytes.get(from + 1..next).unwrap_or_default() {
        let Some(digit) = (byte as char).to_digit(16) else {
            break;
        };
        value = value * 16 + digit;
        digits += 1;
    }
    (digits > 0 && value == u32::from(b'-')).then_some(next)
}

/// Normalize declaration punctuation, leaving the complete authored value
/// untouched. The caller owns the rule boundary and final optional semicolon.
pub(super) fn write(output: &mut String, source: &str) -> bool {
    let Some(colon) = colon(source) else {
        return false;
    };
    output.push_str(source.get(..colon).unwrap_or_default().trim_end());
    let value = source.get(colon + 1..).unwrap_or_default();
    // A whitespace-only custom value is a real token, unlike an empty value.
    // Preserve that exact distinction instead of inserting a new value token.
    if value.trim().is_empty() && starts_custom_value(source) {
        output.push(':');
        output.push_str(value);
        output.push(';');
        return true;
    }
    output.push_str(": ");
    output.push_str(value.trim());
    output.push(';');
    true
}
