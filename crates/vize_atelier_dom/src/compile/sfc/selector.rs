pub(super) fn s2_sfc_fast_path_supported_source(source: &str) -> bool {
    !source_contains_non_void_native_self_closing_tag(source)
}

fn source_contains_non_void_native_self_closing_tag(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut tags = Vec::new();
    let mut index = 0;

    while let Some(tag_start) = find_byte(bytes, index, b'<') {
        let name_start = tag_start + 1;
        if name_start >= bytes.len() {
            break;
        }

        if bytes[name_start] == b'/' {
            let closing_name_start = name_start + 1;
            let closing_name_end = scan_tag_name(bytes, closing_name_start);
            if closing_name_end > closing_name_start {
                pop_closed_tag(&mut tags, &source[closing_name_start..closing_name_end]);
                index = scan_tag_end(bytes, closing_name_end);
                continue;
            }
        }

        if matches!(bytes[name_start], b'!' | b'?') {
            index = scan_special_tag_end(bytes, name_start);
            continue;
        }

        let name_end = scan_tag_name(bytes, name_start);
        if name_end == name_start {
            index = name_start + 1;
            continue;
        }

        let name = &source[name_start..name_end];
        let namespace = tag_namespace(name, tags.last().copied());
        let tag_end = scan_tag_end(bytes, name_end);
        let self_closing = tag_closes_self_closing(bytes, name_end, tag_end);
        if namespace == SourceNamespace::Html
            && is_plain_native_html_tag_name(name)
            && !is_html_void_tag_name(name)
            && !is_allowed_self_closing_special_tag_name(name)
            && self_closing
        {
            return true;
        }

        if namespace == SourceNamespace::Html && !self_closing && is_html_raw_text_tag_name(name) {
            if let Some(raw_text_end) = scan_raw_text_end(bytes, tag_end, name) {
                index = raw_text_end;
                continue;
            }

            return true;
        }

        if !self_closing {
            tags.push(SourceOpenTag { name, namespace });
        }

        index = tag_end;
    }

    false
}

fn find_byte(bytes: &[u8], start: usize, needle: u8) -> Option<usize> {
    bytes[start..]
        .iter()
        .position(|byte| *byte == needle)
        .map(|offset| start + offset)
}

fn pop_closed_tag(tags: &mut Vec<SourceOpenTag<'_>>, name: &str) {
    let Some(tag) = tags.last() else {
        return;
    };
    let closes = match tag.namespace {
        SourceNamespace::Html => tag.name.eq_ignore_ascii_case(name),
        SourceNamespace::Svg | SourceNamespace::MathMl => tag.name == name,
    };
    if closes {
        tags.pop();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceNamespace {
    Html,
    Svg,
    MathMl,
}

#[derive(Clone, Copy)]
struct SourceOpenTag<'source> {
    name: &'source str,
    namespace: SourceNamespace,
}

fn tag_namespace(tag: &str, parent: Option<SourceOpenTag<'_>>) -> SourceNamespace {
    match parent {
        None => html_child_namespace(tag),
        Some(parent) if parent.namespace == SourceNamespace::Html => html_child_namespace(tag),
        Some(parent)
            if parent.namespace == SourceNamespace::Svg
                && is_svg_html_integration_point(parent.name) =>
        {
            html_child_namespace(tag)
        }
        Some(parent)
            if parent.namespace == SourceNamespace::MathMl
                && is_mathml_html_integration_point(parent.name) =>
        {
            html_child_namespace(tag)
        }
        Some(parent) => parent.namespace,
    }
}

fn html_child_namespace(tag: &str) -> SourceNamespace {
    if tag.eq_ignore_ascii_case("svg") {
        SourceNamespace::Svg
    } else if tag.eq_ignore_ascii_case("math") {
        SourceNamespace::MathMl
    } else {
        SourceNamespace::Html
    }
}

fn is_svg_html_integration_point(tag: &str) -> bool {
    matches!(tag, "foreignObject" | "desc" | "title")
}

fn is_mathml_html_integration_point(tag: &str) -> bool {
    matches!(tag, "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext")
}

fn scan_tag_name(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while end < bytes.len() && is_tag_name_byte(bytes[end]) {
        end += 1;
    }
    end
}

fn is_tag_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
}

fn is_plain_native_html_tag_name(name: &str) -> bool {
    name.bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn is_html_void_tag_name(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn is_allowed_self_closing_special_tag_name(name: &str) -> bool {
    matches!(name, "component" | "slot")
}

fn is_html_raw_text_tag_name(name: &str) -> bool {
    matches!(
        name,
        "script"
            | "style"
            | "textarea"
            | "title"
            | "iframe"
            | "noscript"
            | "xmp"
            | "listing"
            | "plaintext"
    )
}

fn scan_raw_text_end(bytes: &[u8], start: usize, name: &str) -> Option<usize> {
    if name == "plaintext" {
        return Some(bytes.len());
    }

    let mut index = start;
    while let Some(tag_start) = find_byte(bytes, index, b'<') {
        let closing_start = tag_start + 1;
        if bytes.get(closing_start) == Some(&b'/') {
            let name_start = closing_start + 1;
            let name_end = name_start + name.len();
            if raw_text_closing_name_matches(bytes, name_start, name_end, name) {
                return Some(scan_tag_end(bytes, name_end));
            }
        }

        index = tag_start + 1;
    }

    None
}

fn raw_text_closing_name_matches(
    bytes: &[u8],
    name_start: usize,
    name_end: usize,
    name: &str,
) -> bool {
    bytes
        .get(name_start..name_end)
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name.as_bytes()))
        && bytes
            .get(name_end)
            .is_none_or(|byte| !is_tag_name_byte(*byte))
}

fn scan_special_tag_end(bytes: &[u8], start: usize) -> usize {
    if bytes.get(start..start + 3) == Some(b"!--") {
        return scan_comment_end(bytes, start + 3);
    }

    scan_tag_end(bytes, start + 1)
}

fn scan_comment_end(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    while index + 2 < bytes.len() {
        if &bytes[index..index + 3] == b"-->" {
            return index + 3;
        }
        index += 1;
    }
    bytes.len()
}

fn scan_tag_end(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    let mut quote = None;

    while index < bytes.len() {
        let byte = bytes[index];

        if let Some(quote_byte) = quote {
            if byte == quote_byte {
                quote = None;
            }
            index += 1;
            continue;
        }

        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'>' => return index + 1,
            _ => {}
        }

        index += 1;
    }

    bytes.len()
}

fn tag_closes_self_closing(bytes: &[u8], start: usize, end: usize) -> bool {
    let mut index = start;
    let mut quote = None;
    let mut last_non_whitespace = None;

    while index < end {
        let byte = bytes[index];

        if let Some(quote_byte) = quote {
            if byte == quote_byte {
                quote = None;
            }
            index += 1;
            continue;
        }

        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'>' => return last_non_whitespace == Some(b'/'),
            b if b.is_ascii_whitespace() => {}
            _ => last_non_whitespace = Some(byte),
        }

        index += 1;
    }

    false
}
