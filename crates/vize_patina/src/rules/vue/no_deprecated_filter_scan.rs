pub(super) fn is_arrow_at(bytes: &[u8], index: usize) -> bool {
    bytes.get(index) == Some(&b'=') && bytes.get(index + 1) == Some(&b'>')
}

/// Index of `=>` after a return-type colon, if the type actually ends in an arrow.
pub(super) fn find_arrow_after_type(bytes: &[u8], mut i: usize) -> Option<usize> {
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    let mut angle = 0i32;
    while i < bytes.len() {
        let Some(&byte) = bytes.get(i) else {
            break;
        };
        match byte {
            b'\'' | b'"' => {
                i = skip_string(bytes, i, byte);
                continue;
            }
            b'`' => {
                i = skip_template(bytes, i);
                continue;
            }
            b'(' => paren += 1,
            b')' => {
                if paren == 0 {
                    return None;
                }
                paren -= 1;
            }
            b'[' => bracket += 1,
            b']' if bracket > 0 => bracket -= 1,
            b'{' => brace += 1,
            b'}' if brace > 0 => brace -= 1,
            b'<' => angle += 1,
            b'>' if angle > 0 => angle -= 1,
            b'=' if paren == 0
                && bracket == 0
                && brace == 0
                && angle == 0
                && bytes.get(i + 1) == Some(&b'>') =>
            {
                return Some(i);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Record each top-level `:` annotation inside an arrow parameter list.
pub(super) fn push_param_type_spans(
    bytes: &[u8],
    mut i: usize,
    end: usize,
    spans: &mut Vec<(usize, usize)>,
) {
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    let mut angle = 0i32;
    while i < end {
        let Some(&byte) = bytes.get(i) else {
            break;
        };
        match byte {
            b'\'' | b'"' => {
                i = skip_string(bytes, i, byte).min(end);
                continue;
            }
            b'`' => {
                i = skip_template(bytes, i).min(end);
                continue;
            }
            b':' if paren == 0 && bracket == 0 && brace == 0 && angle == 0 => {
                let type_start = i + 1;
                i += 1;
                let mut type_paren = 0i32;
                let mut type_bracket = 0i32;
                let mut type_brace = 0i32;
                let mut type_angle = 0i32;
                while i < end {
                    let Some(&byte) = bytes.get(i) else {
                        break;
                    };
                    match byte {
                        b'\'' | b'"' => {
                            i = skip_string(bytes, i, byte).min(end);
                            continue;
                        }
                        b'`' => {
                            i = skip_template(bytes, i).min(end);
                            continue;
                        }
                        b'(' => type_paren += 1,
                        b')' => {
                            if type_paren == 0 {
                                break;
                            }
                            type_paren -= 1;
                        }
                        b'[' => type_bracket += 1,
                        b']' if type_bracket > 0 => type_bracket -= 1,
                        b'{' => type_brace += 1,
                        b'}' if type_brace > 0 => type_brace -= 1,
                        b'<' => type_angle += 1,
                        b'>' if type_angle > 0 => type_angle -= 1,
                        b',' | b'='
                            if type_paren == 0
                                && type_bracket == 0
                                && type_brace == 0
                                && type_angle == 0 =>
                        {
                            break;
                        }
                        _ => {}
                    }
                    i += 1;
                }
                spans.push((type_start, i));
                continue;
            }
            b'(' => paren += 1,
            b')' if paren > 0 => paren -= 1,
            b'[' => bracket += 1,
            b']' if bracket > 0 => bracket -= 1,
            b'{' => brace += 1,
            b'}' if brace > 0 => brace -= 1,
            b'<' => angle += 1,
            b'>' if angle > 0 => angle -= 1,
            _ => {}
        }
        i += 1;
    }
}

/// Returns whether a `/` at this position starts a regex literal, based on the
/// previous significant byte. A regex can begin at the start of the expression
/// or after an operator/opening bracket, but not after a value (identifier,
/// number, `)`, `]`, etc.) where `/` means division.
pub(super) fn regex_allowed(prev: u8) -> bool {
    match prev {
        // No preceding token: start of expression.
        0 => true,
        // After a closing bracket / paren or a word char or `$`, `/` is division.
        b')' | b']' | b'}' => false,
        _ => !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$'),
    }
}

/// Advance past a `'`/`"` string literal starting at the opening quote `i`.
/// Returns the index just past the closing quote (or end of input).
pub(super) fn skip_string(bytes: &[u8], i: usize, quote: u8) -> usize {
    let len = bytes.len();
    let mut j = i + 1;
    while let Some(&byte) = bytes.get(j) {
        match byte {
            b'\\' => j += 2,
            c if c == quote => return j + 1,
            _ => j += 1,
        }
    }
    len
}

/// Advance past a template literal starting at the backtick `i`. Nested `${ … }`
/// interpolations are skipped with brace counting so a `|` inside `${a|b}` is
/// also ignored (template-literal contents are opaque to filter detection).
pub(super) fn skip_template(bytes: &[u8], i: usize) -> usize {
    let len = bytes.len();
    let mut j = i + 1;
    while let Some(&byte) = bytes.get(j) {
        match byte {
            b'\\' => j += 2,
            b'`' => return j + 1,
            b'$' if bytes.get(j + 1) == Some(&b'{') => {
                // Skip the balanced `${ … }` interpolation block.
                let mut depth = 1;
                j += 2;
                while let Some(&byte) = bytes.get(j)
                    && depth > 0
                {
                    match byte {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
            }
            _ => j += 1,
        }
    }
    len
}

/// Advance past a regex literal starting at the `/` at `i`. Character classes
/// `[ … ]` are honoured so a `/` inside them does not end the literal early.
pub(super) fn skip_regex(bytes: &[u8], i: usize) -> usize {
    let len = bytes.len();
    let mut j = i + 1;
    let mut in_class = false;
    while let Some(&byte) = bytes.get(j) {
        match byte {
            b'\\' => j += 2,
            b'[' => {
                in_class = true;
                j += 1;
            }
            b']' => {
                in_class = false;
                j += 1;
            }
            b'/' if !in_class => return j + 1,
            _ => j += 1,
        }
    }
    len
}
