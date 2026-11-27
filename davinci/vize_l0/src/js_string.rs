//! JavaScript string literals shared by compiler-owned producers.

use crate::String;
use crate::string_builder::TextBuf;

/// The two text buffers a literal is appended to.
trait LiteralSink {
    fn push_str(&mut self, text: &str);
    fn push(&mut self, character: char);
}

impl LiteralSink for String {
    #[inline(always)]
    fn push_str(&mut self, text: &str) {
        String::push_str(self, text);
    }

    #[inline(always)]
    fn push(&mut self, character: char) {
        String::push(self, character);
    }
}

impl LiteralSink for TextBuf {
    #[inline(always)]
    fn push_str(&mut self, text: &str) {
        TextBuf::push_str(self, text);
    }

    #[inline(always)]
    fn push(&mut self, character: char) {
        TextBuf::push(self, character);
    }
}

/// Append one double-quoted JavaScript string literal.
pub fn push_js_string_literal(output: &mut String, value: &str) {
    write_js_string_literal(output, value);
}

/// Append one double-quoted JavaScript string literal to a [`TextBuf`].
pub fn push_js_string_literal_text(output: &mut TextBuf, value: &str) {
    write_js_string_literal(output, value);
}

#[inline(always)]
fn write_js_string_literal(output: &mut impl LiteralSink, value: &str) {
    output.push('"');

    let mut segment_start = 0usize;
    for (index, char) in value.char_indices() {
        let escaped = match char {
            '"' => Some("\\\""),
            '\\' => Some("\\\\"),
            '\n' => Some("\\n"),
            '\r' => Some("\\r"),
            '\t' => Some("\\t"),
            '\u{08}' => Some("\\b"),
            '\u{0c}' => Some("\\f"),
            '\u{00}'..='\u{1f}' => {
                output.push_str(value.get(segment_start..index).unwrap_or_default());
                push_control_escape(output, char as u8);
                segment_start = index + char.len_utf8();
                None
            }
            _ => None,
        };

        if let Some(escaped) = escaped {
            output.push_str(value.get(segment_start..index).unwrap_or_default());
            output.push_str(escaped);
            segment_start = index + char.len_utf8();
        }
    }

    output.push_str(value.get(segment_start..).unwrap_or_default());
    output.push('"');
}

fn push_control_escape(output: &mut impl LiteralSink, byte: u8) {
    output.push_str("\\u00");
    for nibble in [byte >> 4, byte & 0x0f] {
        output.push(char::from_digit(u32::from(nibble), 16).unwrap_or('0'));
    }
}

#[cfg(test)]
mod tests;
