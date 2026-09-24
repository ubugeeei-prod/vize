//! `<p>` auto-closes before a block start tag. A later `</p>` is the
//! legacy parser's fatal `InvalidEndTag`, which drops the inline render.
//! The S2 emitter does not report that error, so the source stays off
//! the SFC fast path.

pub(super) fn source_has_invalid_p_end_tag(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut open_p = 0i32;
    while let Some(start) = find_byte(bytes, index, b'<') {
        let name_start = start + 1;
        let Some(&first) = bytes.get(name_start) else {
            break;
        };
        let closing = first == b'/';
        let raw_start = if closing { name_start + 1 } else { name_start };
        if !closing && matches!(bytes.get(raw_start), Some(b'!' | b'?')) {
            index = name_start + 1;
            continue;
        }
        let name_end = scan_name(bytes, raw_start);
        if name_end == raw_start {
            index = raw_start + 1;
            continue;
        }
        let name = source.get(raw_start..name_end).unwrap_or_default();
        if closing {
            if name.eq_ignore_ascii_case("p") {
                if open_p == 0 {
                    return true;
                }
                open_p -= 1;
            }
        } else if name.eq_ignore_ascii_case("p") && open_p == 0 {
            open_p = 1;
        } else if closes_open_p(name) && open_p > 0 {
            open_p -= 1;
            if name.eq_ignore_ascii_case("p") {
                open_p += 1;
            }
        }
        index = name_end;
    }
    false
}

fn closes_open_p(name: &str) -> bool {
    const TAGS: &[&str] = &[
        "address",
        "article",
        "aside",
        "blockquote",
        "div",
        "dl",
        "fieldset",
        "footer",
        "form",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "hr",
        "main",
        "nav",
        "ol",
        "p",
        "pre",
        "section",
        "table",
        "ul",
    ];
    TAGS.iter().any(|tag| name.eq_ignore_ascii_case(tag))
}

fn find_byte(bytes: &[u8], start: usize, needle: u8) -> Option<usize> {
    bytes
        .get(start..)?
        .iter()
        .position(|byte| *byte == needle)
        .map(|offset| start + offset)
}

fn scan_name(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    while bytes.get(index).is_some_and(u8::is_ascii_alphanumeric) {
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::source_has_invalid_p_end_tag;

    #[test]
    fn a_block_inside_p_is_an_invalid_end_tag() {
        assert!(source_has_invalid_p_end_tag("<p>Intro <div>x</div></p>"));
        assert!(!source_has_invalid_p_end_tag("<p>Intro</p><div>x</div>"));
    }
}
