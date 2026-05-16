use vize_carton::String;

pub(crate) fn normalized_scheme(value: &str) -> Option<(String, &str)> {
    let mut scheme = String::default();
    let mut saw_scheme_char = false;

    for (index, ch) in value.char_indices() {
        if ch == ':' {
            return saw_scheme_char.then(|| (scheme, &value[index + 1..]));
        }

        if ch.is_ascii_whitespace() || ch.is_ascii_control() {
            continue;
        }

        if matches!(ch, '/' | '#' | '?') {
            return None;
        }

        if ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.') {
            saw_scheme_char = true;
            scheme.push(ch.to_ascii_lowercase());
            continue;
        }

        return None;
    }

    None
}

pub(crate) fn has_javascript_scheme(value: &str) -> bool {
    normalized_scheme(value).is_some_and(|(scheme, _)| scheme.as_str() == "javascript")
}

pub(crate) fn is_executable_data_url(rest: &str) -> bool {
    let media_type = rest
        .trim_start_matches(|ch: char| ch.is_ascii_whitespace() || ch.is_ascii_control())
        .split(',')
        .next()
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();

    matches!(
        media_type.as_str(),
        "text/html" | "application/xhtml+xml" | "image/svg+xml" | "text/xml" | "application/xml"
    )
}

pub(crate) fn is_unsafe_url(value: &str) -> bool {
    let Some((scheme, rest)) = normalized_scheme(value) else {
        return false;
    };

    match scheme.as_str() {
        "javascript" | "vbscript" => true,
        "data" => is_executable_data_url(rest),
        _ => false,
    }
}
