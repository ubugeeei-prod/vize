//! HTML text and attribute escaping shared by the Vapor emitters.

use vize_carton::String;

pub(crate) fn escape_html_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    let mut start = 0;
    for (offset, byte) in text.bytes().enumerate() {
        let entity = match byte {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&quot;",
            b'\'' => "&#39;",
            _ => continue,
        };
        if start < offset {
            escaped.push_str(&text[start..offset]);
        }
        escaped.push_str(entity);
        start = offset + 1;
    }
    escaped.push_str(&text[start..]);
    escaped
}
