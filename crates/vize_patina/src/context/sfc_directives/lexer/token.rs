//! Token-boundary helpers for the script directive lexer.

pub(super) fn ends_with_unescaped_backslash(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .rev()
        .take_while(|&&byte| byte == b'\\')
        .count()
        % 2
        == 1
}

pub(super) fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') || byte >= 0x80
}

pub(super) fn identifier_end(bytes: &[u8], start: usize) -> usize {
    bytes[start..]
        .iter()
        .position(|byte| {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') || *byte >= 0x80)
        })
        .map_or(bytes.len(), |relative| start + relative)
}

pub(super) fn identifier_allows_expression(identifier: &[u8]) -> bool {
    matches!(
        identifier,
        b"await"
            | b"case"
            | b"delete"
            | b"do"
            | b"else"
            | b"in"
            | b"instanceof"
            | b"new"
            | b"of"
            | b"return"
            | b"throw"
            | b"typeof"
            | b"void"
            | b"yield"
    )
}

pub(super) fn identifier_opens_control_paren(identifier: &[u8]) -> bool {
    matches!(
        identifier,
        b"catch" | b"for" | b"if" | b"switch" | b"while" | b"with"
    )
}

pub(super) fn is_jsx_start(bytes: &[u8], start: usize, tsx: bool) -> bool {
    let Some(next) = bytes.get(start + 1).copied() else {
        return false;
    };
    (next == b'>' || is_identifier_start(next)) && !(tsx && is_generic_arrow_start(bytes, start))
}

fn is_generic_arrow_start(bytes: &[u8], start: usize) -> bool {
    const LOOKAHEAD_LIMIT: usize = 4 * 1024;

    let bytes = &bytes[..bytes.len().min(start.saturating_add(LOOKAHEAD_LIMIT))];
    let Some(type_end) = matching_close(bytes, start, b'<', b'>') else {
        return false;
    };
    let parameters_start = skip_trivia(bytes, type_end + 1);
    if bytes.get(parameters_start) != Some(&b'(') {
        return false;
    }
    let Some(parameters_end) = matching_close(bytes, parameters_start, b'(', b')') else {
        // TSX accepts a generic arrow before its closing `) =>` is visible when
        // the type-parameter grammar itself disambiguates it from JSX.
        return disambiguates_type_parameters(&bytes[start + 1..type_end]);
    };
    let arrow_start = skip_trivia(bytes, parameters_end + 1);
    has_arrow_after_parameters(bytes, arrow_start)
}

fn has_arrow_after_parameters(bytes: &[u8], start: usize) -> bool {
    if bytes
        .get(start..)
        .is_some_and(|remaining| remaining.starts_with(b"=>"))
    {
        return true;
    }
    if bytes.get(start) != Some(&b':') {
        return false;
    }

    let mut depths = [0usize; 4];
    let mut quote = None;
    let mut line_comment = false;
    let mut block_comment = false;
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        let next = bytes.get(cursor + 1).copied();
        if line_comment {
            line_comment = byte != b'\n';
        } else if block_comment {
            if (byte, next) == (b'*', Some(b'/')) {
                block_comment = false;
                cursor += 1;
            }
        } else if let Some(active_quote) = quote {
            match byte {
                b'\\' => cursor += 1,
                byte if byte == active_quote => quote = None,
                _ => {}
            }
        } else {
            let at_top_level = depths.iter().all(|depth| *depth == 0);
            match (byte, next) {
                (b'=', Some(b'>')) if at_top_level => return true,
                (b';', _) if at_top_level => return false,
                (b'/', Some(b'/')) => {
                    line_comment = true;
                    cursor += 1;
                }
                (b'/', Some(b'*')) => {
                    block_comment = true;
                    cursor += 1;
                }
                (b'\'' | b'"' | b'`', _) => quote = Some(byte),
                (b'<', _) => depths[0] += 1,
                (b'>', _) => depths[0] = depths[0].saturating_sub(1),
                (b'(', _) => depths[1] += 1,
                (b')', _) => depths[1] = depths[1].saturating_sub(1),
                (b'[', _) => depths[2] += 1,
                (b']', _) => depths[2] = depths[2].saturating_sub(1),
                (b'{', _) => depths[3] += 1,
                (b'}', _) => depths[3] = depths[3].saturating_sub(1),
                _ => {}
            }
        }
        cursor += 1;
    }
    false
}

fn disambiguates_type_parameters(bytes: &[u8]) -> bool {
    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut quote = None;
    let mut cursor = 0;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if let Some(active_quote) = quote {
            match byte {
                b'\\' => cursor += 1,
                byte if byte == active_quote => quote = None,
                _ => {}
            }
        } else {
            let at_top_level =
                angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
            match byte {
                b'\'' | b'"' | b'`' => quote = Some(byte),
                b',' | b'=' if at_top_level => return true,
                byte if at_top_level && is_identifier_start(byte) => {
                    let end = identifier_end(bytes, cursor);
                    if matches!(&bytes[cursor..end], b"const" | b"extends") {
                        return true;
                    }
                    cursor = end - 1;
                }
                b'<' => angle_depth += 1,
                b'>' => angle_depth = angle_depth.saturating_sub(1),
                b'(' => paren_depth += 1,
                b')' => paren_depth = paren_depth.saturating_sub(1),
                b'[' => bracket_depth += 1,
                b']' => bracket_depth = bracket_depth.saturating_sub(1),
                b'{' => brace_depth += 1,
                b'}' => brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }
        }
        cursor += 1;
    }
    false
}

fn matching_close(bytes: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut line_comment = false;
    let mut block_comment = false;
    let mut cursor = start;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        let next = bytes.get(cursor + 1).copied();
        if line_comment {
            if byte == b'\n' {
                line_comment = false;
            }
        } else if block_comment {
            if (byte, next) == (b'*', Some(b'/')) {
                block_comment = false;
                cursor += 1;
            }
        } else if let Some(active_quote) = quote {
            match byte {
                b'\\' => cursor += 1,
                byte if byte == active_quote => quote = None,
                _ => {}
            }
        } else {
            match byte {
                b'/' if next == Some(b'/') => {
                    line_comment = true;
                    cursor += 1;
                }
                b'/' if next == Some(b'*') => {
                    block_comment = true;
                    cursor += 1;
                }
                b'\'' | b'"' | b'`' => quote = Some(byte),
                byte if byte == open => depth += 1,
                byte if byte == close
                    && !(close == b'>' && bytes.get(cursor.wrapping_sub(1)) == Some(&b'=')) =>
                {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(cursor);
                    }
                }
                _ => {}
            }
        }
        cursor += 1;
    }
    None
}

fn skip_whitespace(bytes: &[u8], start: usize) -> usize {
    start
        + bytes[start..]
            .iter()
            .take_while(|byte| byte.is_ascii_whitespace())
            .count()
}

fn skip_trivia(bytes: &[u8], start: usize) -> usize {
    let mut cursor = skip_whitespace(bytes, start);
    loop {
        match (bytes.get(cursor), bytes.get(cursor + 1)) {
            (Some(b'/'), Some(b'/')) => {
                cursor += 2;
                cursor += bytes[cursor..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .unwrap_or(bytes.len() - cursor);
            }
            (Some(b'/'), Some(b'*')) => {
                let Some(end) = bytes[cursor + 2..]
                    .windows(2)
                    .position(|candidate| candidate == b"*/")
                else {
                    return bytes.len();
                };
                cursor += end + 4;
            }
            _ => return cursor,
        }
        cursor = skip_whitespace(bytes, cursor);
    }
}
