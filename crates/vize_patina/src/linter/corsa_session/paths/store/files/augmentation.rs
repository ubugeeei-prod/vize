/// A worker that classified one huge file must not keep that allocation.
const RETAINED_BUFFER_LIMIT: usize = 64 * 1024;

pub(super) fn path_is_module_augmentation(path: &std::path::Path) -> bool {
    thread_local! {
        static BUFFER: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.clear();
        let found = augmentation_bytes_match(path, &mut buffer);
        if buffer.capacity() > RETAINED_BUFFER_LIMIT {
            buffer.clear();
            buffer.shrink_to(RETAINED_BUFFER_LIMIT);
        }
        found
    })
}

fn augmentation_bytes_match(path: &std::path::Path, buffer: &mut Vec<u8>) -> bool {
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    if std::io::Read::read_to_end(&mut file, buffer).is_err() {
        return false;
    }
    // The exact single-space needles are the historical reject. A newline
    // between `declare` and `module` is not a hit, matching `str::contains`.
    if memchr::memmem::find(buffer.as_slice(), b"declare module").is_none()
        && memchr::memmem::find(buffer.as_slice(), b"declare global").is_none()
    {
        return false;
    }
    let Ok(content) = std::str::from_utf8(buffer.as_slice()) else {
        return false;
    };
    is_module_augmentation(content)
}

fn is_module_augmentation(content: &str) -> bool {
    if !content.contains("declare module") && !content.contains("declare global") {
        return false;
    }
    has_top_level_import_or_export(content) && has_top_level_declare_augmentation(content)
}

fn has_top_level_import_or_export(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut index = 0;
    let mut brace_depth = 0usize;
    let mut at_statement_start = true;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index + 2);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index + 2);
            }
            b'\'' | b'"' | b'`' => index = skip_quoted(bytes, index),
            b'{' => {
                brace_depth += 1;
                at_statement_start = false;
                index += 1;
            }
            b'}' => {
                brace_depth = brace_depth.saturating_sub(1);
                at_statement_start = brace_depth == 0;
                index += 1;
            }
            b';' if brace_depth == 0 => {
                at_statement_start = true;
                index += 1;
            }
            byte if byte.is_ascii_whitespace() => index += 1,
            _ if brace_depth == 0 && at_statement_start => {
                if starts_with_statement_keyword(bytes, index, b"import")
                    || starts_with_statement_keyword(bytes, index, b"export")
                {
                    return true;
                }
                at_statement_start = false;
                index = skip_token(bytes, index);
            }
            _ => index += 1,
        }
    }
    false
}

fn has_top_level_declare_augmentation(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut index = 0;
    let mut brace_depth = 0usize;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index + 2);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index + 2);
            }
            b'\'' | b'"' | b'`' => index = skip_quoted(bytes, index),
            b'{' => {
                brace_depth += 1;
                index += 1;
            }
            b'}' => {
                brace_depth = brace_depth.saturating_sub(1);
                index += 1;
            }
            _ if brace_depth == 0 && starts_with_keyword(bytes, index, b"declare") => {
                let scope = skip_trivia(bytes, index + b"declare".len());
                if starts_with_keyword(bytes, scope, b"global") {
                    return true;
                }
                if starts_with_keyword(bytes, scope, b"module") {
                    let name = skip_trivia(bytes, scope + b"module".len());
                    if bytes
                        .get(name)
                        .is_some_and(|byte| matches!(byte, b'\'' | b'"'))
                    {
                        return true;
                    }
                }
                index = skip_token(bytes, index);
            }
            _ => index = skip_token(bytes, index),
        }
    }
    false
}

fn starts_with_statement_keyword(bytes: &[u8], index: usize, keyword: &[u8]) -> bool {
    if !starts_with_keyword(bytes, index, keyword) {
        return false;
    }
    let after = index + keyword.len();
    match keyword {
        b"import" => bytes.get(after).is_some_and(|byte| {
            byte.is_ascii_whitespace() || matches!(byte, b'{' | b'*' | b'\'' | b'"')
        }),
        b"export" => bytes
            .get(after)
            .is_some_and(|byte| byte.is_ascii_whitespace() || matches!(byte, b'{' | b'*' | b'=')),
        _ => false,
    }
}

fn starts_with_keyword(bytes: &[u8], index: usize, keyword: &[u8]) -> bool {
    bytes
        .get(index..)
        .is_some_and(|rest| rest.starts_with(keyword))
        && bytes
            .get(index + keyword.len())
            .is_none_or(|byte| !is_identifier_continue(*byte))
}

fn skip_trivia(bytes: &[u8], mut index: usize) -> usize {
    loop {
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        if bytes.get(index..index + 2) == Some(b"//") {
            index = skip_line_comment(bytes, index + 2);
        } else if bytes.get(index..index + 2) == Some(b"/*") {
            index = skip_block_comment(bytes, index + 2);
        } else {
            return index;
        }
    }
}

fn skip_line_comment(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes.get(index) != Some(&b'\n') {
        index += 1;
    }
    index
}

fn skip_block_comment(bytes: &[u8], mut index: usize) -> usize {
    while index + 1 < bytes.len()
        && !(bytes.get(index) == Some(&b'*') && bytes.get(index + 1) == Some(&b'/'))
    {
        index += 1;
    }
    (index + 2).min(bytes.len())
}

fn skip_quoted(bytes: &[u8], mut index: usize) -> usize {
    let Some(&quote) = bytes.get(index) else {
        return bytes.len();
    };
    index += 1;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\\' => index = (index + 2).min(bytes.len()),
            byte if byte == quote => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

fn skip_token(bytes: &[u8], mut index: usize) -> usize {
    if !bytes
        .get(index)
        .is_some_and(|byte| is_identifier_start(*byte))
    {
        return index + 1;
    }
    index += 1;
    while bytes
        .get(index)
        .is_some_and(|byte| is_identifier_continue(*byte))
    {
        index += 1;
    }
    index
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}
