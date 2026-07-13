use vize_carton::String;

pub(super) fn comma(output: &mut String, first: &mut bool) {
    if *first {
        *first = false;
    } else {
        output.push_str(", ");
    }
}

pub(super) fn quote(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{2028}' => output.push_str("\\u2028"),
            '\u{2029}' => output.push_str("\\u2029"),
            character => output.push(character),
        }
    }
    output.push('"');
}
