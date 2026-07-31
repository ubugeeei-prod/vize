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

pub(super) fn is_jsx_start(bytes: &[u8], start: usize) -> bool {
    let Some(next) = bytes.get(start + 1).copied() else {
        return false;
    };
    (next == b'>' || is_identifier_start(next)) && !is_generic_arrow_start(bytes, start)
}

fn is_generic_arrow_start(bytes: &[u8], start: usize) -> bool {
    let Some(type_end) = matching_close(bytes, start, b'<', b'>') else {
        return false;
    };
    let parameters_start = skip_whitespace(bytes, type_end + 1);
    if bytes.get(parameters_start) != Some(&b'(') {
        return false;
    }
    let Some(parameters_end) = matching_close(bytes, parameters_start, b'(', b')') else {
        // `<T,>(` is the standard TSX spelling that disambiguates a generic
        // arrow from JSX. The parameter list may continue on later lines, so
        // the current line cannot always contain its closing `)` and `=>`.
        return bytes[start + 1..type_end]
            .iter()
            .rfind(|byte| !byte.is_ascii_whitespace())
            == Some(&b',');
    };
    bytes[parameters_end + 1..]
        .windows(2)
        .any(|candidate| candidate == b"=>")
}

fn matching_close(bytes: &[u8], start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut cursor = start;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if let Some(active_quote) = quote {
            match byte {
                b'\\' => cursor += 1,
                byte if byte == active_quote => quote = None,
                _ => {}
            }
        } else {
            match byte {
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
