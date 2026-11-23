use super::*;

#[test]
fn escapes_js_string_literal_controls() {
    let mut output = String::default();
    push_js_string_literal(&mut output, "a\"b\\c\n\u{01}");
    assert_eq!(output.as_str(), r#""a\"b\\c\n\u0001""#);
}

#[test]
fn ascii_between_original_segments() {
    for byte in 0..=127 {
        let input = [b'Z', byte, b'Q'];
        let value = core::str::from_utf8(&input).unwrap();
        let mut output = String::from("P");
        push_js_string_literal(&mut output, value);
        let literal = output.strip_prefix('P').unwrap();
        assert!(literal.starts_with("\"Z") && literal.ends_with("Q\""));
        assert_eq!(
            serde_json::from_str::<Box<str>>(literal).unwrap().as_ref(),
            value
        );
    }
}
