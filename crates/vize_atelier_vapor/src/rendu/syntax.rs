use vize_carton::String;

pub(super) fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .into()
}

pub(super) fn escape_html_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .into()
}

pub(super) fn escape_html_comment(value: &str) -> String {
    let mut escaped = value.replace("--", "- -");
    if escaped.ends_with('-') {
        escaped.push(' ');
    }
    escaped.into()
}
