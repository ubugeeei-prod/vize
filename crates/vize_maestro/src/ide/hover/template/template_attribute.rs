pub(crate) fn name_at_offset(content: &str, offset: usize) -> Option<&str> {
    let cursor = offset.min(content.len());
    let tag_start = content[..cursor].rfind('<')?;
    let bytes = content.as_bytes();
    if matches!(bytes.get(tag_start + 1), Some(b'/' | b'!' | b'?')) {
        return None;
    }

    let tag_end = find_open_tag_end(content, tag_start)?;
    if cursor > tag_end {
        return None;
    }

    let mut pos = tag_start + 1;
    while pos < tag_end {
        let byte = bytes[pos];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            pos += 1;
        } else {
            break;
        }
    }

    while pos < tag_end {
        while pos < tag_end && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= tag_end || matches!(bytes[pos], b'/' | b'>') {
            break;
        }

        let attr_start = pos;
        while pos < tag_end
            && !bytes[pos].is_ascii_whitespace()
            && !matches!(bytes[pos], b'=' | b'/' | b'>')
        {
            pos += 1;
        }
        let attr_end = pos;
        if attr_start == attr_end {
            return None;
        }

        if cursor >= attr_start && cursor <= attr_end {
            return Some(&content[attr_start..attr_end]);
        }

        while pos < tag_end && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos < tag_end && bytes[pos] == b'=' {
            pos += 1;
            while pos < tag_end && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos < tag_end && matches!(bytes[pos], b'"' | b'\'') {
                let quote = bytes[pos];
                pos += 1;
                while pos < tag_end && bytes[pos] != quote {
                    pos += 1;
                }
                if pos < tag_end {
                    pos += 1;
                }
            } else {
                while pos < tag_end && !bytes[pos].is_ascii_whitespace() && bytes[pos] != b'>' {
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
        let ch = content[pos..].chars().next()?;
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
