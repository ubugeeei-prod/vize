pub(crate) fn name_at_offset(content: &str, offset: usize) -> Option<&str> {
    let cursor = offset.min(content.len());
    let tag_start = content.get(..cursor).and_then(|head| head.rfind('<'))?;
    let bytes = content.as_bytes();
    if matches!(bytes.get(tag_start + 1), Some(b'/' | b'!' | b'?')) {
        return None;
    }

    let tag_end = find_open_tag_end(content, tag_start)?;
    if cursor > tag_end {
        return None;
    }

    // Byte at `i` while still inside the tag; `None` past `tag_end`.
    let at = |i: usize| {
        if i < tag_end {
            bytes.get(i).copied()
        } else {
            None
        }
    };
    let mut pos = tag_start + 1;
    while at(pos).is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')) {
        pos += 1;
    }

    while pos < tag_end {
        while at(pos).is_some_and(|byte| byte.is_ascii_whitespace()) {
            pos += 1;
        }
        if matches!(at(pos), None | Some(b'/' | b'>')) {
            break;
        }

        let attr_start = pos;
        while at(pos)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'/' | b'>'))
        {
            pos += 1;
        }
        let attr_end = pos;
        if attr_start == attr_end {
            return None;
        }

        if cursor >= attr_start && cursor <= attr_end {
            return content.get(attr_start..attr_end);
        }

        while at(pos).is_some_and(|byte| byte.is_ascii_whitespace()) {
            pos += 1;
        }
        if at(pos) == Some(b'=') {
            pos += 1;
            while at(pos).is_some_and(|byte| byte.is_ascii_whitespace()) {
                pos += 1;
            }
            if let Some(quote @ (b'"' | b'\'')) = at(pos) {
                pos += 1;
                while at(pos).is_some_and(|byte| byte != quote) {
                    pos += 1;
                }
                if pos < tag_end {
                    pos += 1;
                }
            } else {
                while at(pos).is_some_and(|byte| !byte.is_ascii_whitespace() && byte != b'>') {
                    pos += 1;
                }
            }
        }
    }

    None
}

fn find_open_tag_end(content: &str, tag_start: usize) -> Option<usize> {
    let mut quote = None;
    let mut pos = tag_start;

    while pos < content.len() {
        let ch = content.get(pos..).and_then(|rest| rest.chars().next())?;
        if let Some(open_quote) = quote {
            if ch == open_quote {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '>' {
            return Some(pos);
        }
        pos += ch.len_utf8();
    }

    None
}
