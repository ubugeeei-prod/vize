pub(super) fn outside_prefix_is_safe(prefix: &str) -> bool {
    match prefix.find('\n') {
        None => is_whitespace_only(prefix),
        Some(newline) => {
            prefix.get(..newline).is_some_and(is_comment_trivia)
                && prefix.get(newline + 1..).is_some_and(is_whitespace_only)
        }
    }
}

pub(super) fn is_comment_trivia(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes.get(index).copied() {
            Some(b' ' | b'\t' | b'\n' | b'\r') => index += 1,
            Some(b'/') if bytes.get(index + 1) == Some(&b'/') => {
                index += 2;
                while bytes.get(index).is_some_and(|byte| *byte != b'\n') {
                    index += 1;
                }
            }
            Some(b'/') if bytes.get(index + 1) == Some(&b'*') => {
                let Some(relative) = text.get(index..).and_then(|tail| tail.find("*/")) else {
                    return false;
                };
                let body_start = index + 2;
                let body_end = index + relative;
                if text
                    .get(body_start..body_end)
                    .is_none_or(contains_blank_line)
                {
                    return false;
                }
                index = body_end + 2;
            }
            _ => return false,
        }
    }
    true
}

pub(super) fn contains_blank_line(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes.get(index) == Some(&b'\n') {
            let mut next = index + 1;
            while bytes
                .get(next)
                .is_some_and(|byte| matches!(*byte, b' ' | b'\t' | b'\r'))
            {
                next += 1;
            }
            if bytes.get(next) == Some(&b'\n') {
                return true;
            }
        }
        index += 1;
    }
    false
}

pub(super) fn is_whitespace_only(text: &str) -> bool {
    text.bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
}

pub(super) fn line_start(source: &str, index: usize) -> usize {
    source
        .get(..index)
        .and_then(|head| head.rfind('\n'))
        .map_or(0, |found| found + 1)
}

pub(super) fn trim_hspace_start(source: &str, floor: usize, prop_start: usize) -> usize {
    let mut start = prop_start;
    while start > floor && matches!(source.as_bytes().get(start - 1), Some(b' ' | b'\t')) {
        start -= 1;
    }
    start
}

pub(super) fn skip_hspace(source: &str, mut index: usize, limit: usize) -> usize {
    let bytes = source.as_bytes();
    while index < limit && matches!(bytes.get(index), Some(b' ' | b'\t')) {
        index += 1;
    }
    index
}

pub(super) fn skip_line_comment(source: &str, mut index: usize, limit: usize) -> usize {
    while index < limit && source.as_bytes().get(index) != Some(&b'\n') {
        index += 1;
    }
    index
}

pub(super) fn block_comment_end(source: &str, index: usize, limit: usize) -> Option<usize> {
    let tail = source.get(index..limit)?;
    if !tail.starts_with("/*") {
        return None;
    }
    Some(index + tail.find("*/")? + 2)
}
