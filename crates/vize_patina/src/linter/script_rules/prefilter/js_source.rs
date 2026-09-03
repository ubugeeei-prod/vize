use memchr::memmem;

pub(super) fn imports_from_vue(bytes: &[u8]) -> bool {
    contains_keyword_followed_by(bytes, b"from", module_specifier_is_vue)
}

pub(super) fn source_may_define_component_options(bytes: &[u8]) -> bool {
    contains_keyword_followed_by(bytes, b"export", |bytes, index| {
        keyword_at(bytes, index, b"default")
    }) || memmem::find(bytes, b"defineComponent").is_some()
        || memmem::find(bytes, b"Vue.extend").is_some()
}

fn contains_keyword_followed_by(
    bytes: &[u8],
    keyword: &[u8],
    next_matches: impl Fn(&[u8], usize) -> bool,
) -> bool {
    let mut search_start = 0;
    while let Some(relative) = memmem::find(&bytes[search_start..], keyword) {
        let start = search_start + relative;
        let end = start + keyword.len();
        search_start = end;
        if !has_identifier_boundaries(bytes, start, end) {
            continue;
        }
        let Some(next) = skip_js_trivia(bytes, end) else {
            return true;
        };
        if next_matches(bytes, next) {
            return true;
        }
    }
    false
}

fn module_specifier_is_vue(bytes: &[u8], index: usize) -> bool {
    bytes[index..].starts_with(b"'vue'") || bytes[index..].starts_with(b"\"vue\"")
}

fn keyword_at(bytes: &[u8], start: usize, keyword: &[u8]) -> bool {
    let end = start + keyword.len();
    bytes.get(start..end) == Some(keyword) && has_identifier_boundaries(bytes, start, end)
}

fn skip_js_trivia(bytes: &[u8], mut index: usize) -> Option<usize> {
    loop {
        while bytes
            .get(index)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            index += 1;
        }
        if bytes
            .get(index..)
            .is_some_and(|rest| rest.starts_with(b"//"))
        {
            index += 2;
            while let Some(byte) = bytes.get(index) {
                index += 1;
                if *byte == b'\n' || *byte == b'\r' {
                    break;
                }
            }
            continue;
        }
        if bytes
            .get(index..)
            .is_some_and(|rest| rest.starts_with(b"/*"))
        {
            let rest = &bytes[index + 2..];
            let relative_end = memmem::find(rest, b"*/")?;
            index += 2 + relative_end + 2;
            continue;
        }
        return Some(index);
    }
}

fn has_identifier_boundaries(bytes: &[u8], start: usize, end: usize) -> bool {
    !bytes
        .get(start.wrapping_sub(1))
        .is_some_and(|byte| is_ascii_identifier_continue(*byte))
        && !bytes
            .get(end)
            .is_some_and(|byte| is_ascii_identifier_continue(*byte))
}

fn is_ascii_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}
