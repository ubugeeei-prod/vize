use vize_carton::String;

pub(super) fn push_js_string_literal(output: &mut String, value: &str) {
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
                output.push_str(&value[segment_start..index]);
                push_control_escape(output, char as u8);
                segment_start = index + char.len_utf8();
                None
            }
            _ => None,
        };

        if let Some(escaped) = escaped {
            output.push_str(&value[segment_start..index]);
            output.push_str(escaped);
            segment_start = index + char.len_utf8();
        }
    }

    output.push_str(&value[segment_start..]);
    output.push('"');
}

fn push_control_escape(output: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.push_str("\\u00");
    output.push(HEX[(byte >> 4) as usize] as char);
    output.push(HEX[(byte & 0x0f) as usize] as char);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_js_string_literal_controls() {
        let mut output = String::default();
        push_js_string_literal(&mut output, "a\"b\\c\n\u{01}");
        assert_eq!(output.as_str(), r#""a\"b\\c\n\u0001""#);
    }
}
