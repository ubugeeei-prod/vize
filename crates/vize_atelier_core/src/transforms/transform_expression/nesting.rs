/// Maximum expression nesting depth accepted before parsing.
///
/// OXC recurses for nested brackets; stack overflow (#956) and a parser timeout
/// at depth 32 (#2944) cannot be caught, so every entry point shares this guard.
pub const MAX_EXPRESSION_NESTING_DEPTH: usize = 31;

/// Returns the maximum parser-recursion depth in `content`.
///
/// Brackets and unambiguous TypeScript angles are paired, while decorator markers
/// accumulate for OXC's recursive parser. Strings, template text, comments, and
/// regexes are skipped; `${...}` template interpolations are scanned.
fn analyze_expression_nesting(content: &str) -> (usize, bool) {
    let bytes = content.as_bytes();
    let (mut angle_depth, mut decorator_depth) = (0usize, 0usize);
    let mut max_depth = 0usize;
    let mut delimiters = Vec::new();
    let mut delimiters_balanced = true;
    let mut can_start_regex = true;
    let mut template_interpolation_depths = Vec::new();
    // Plain `foo < bar` is indistinguishable from a type argument to a byte
    // scanner. Enter angle-tracking mode only for the repeated structural type
    // prefixes present in the parser-timeout class (`<{`, `<[`), discovered
    // while scanning so strings and comments cannot activate the mode.
    let mut structural_type_angle_opens = 0usize;
    let mut track_type_angles = false;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b' ' | b'\t' | b'\r' | b'\n' => {
                i += 1;
                continue;
            }
            b'"' | b'\'' => {
                i = skip_quoted(bytes, i + 1, b);
                can_start_regex = false;
                continue;
            }
            b'`' => {
                let (next, has_interpolation) = skip_template_text(bytes, i + 1);
                i = next;
                if has_interpolation {
                    delimiters.push(b'}');
                    template_interpolation_depths.push(delimiters.len());
                    let effective_angle_depth = if track_type_angles { angle_depth } else { 0 };
                    max_depth =
                        max_depth.max(delimiters.len() + effective_angle_depth + decorator_depth);
                    can_start_regex = true;
                } else {
                    can_start_regex = false;
                }
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i = skip_line_comment(bytes, i + 2);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = skip_block_comment(bytes, i + 2);
                continue;
            }
            b'/' if can_start_regex => {
                i = skip_regex(bytes, i + 1);
                can_start_regex = false;
                continue;
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                let start = i;
                i = skip_identifier(bytes, i + 1);
                can_start_regex = keyword_allows_regex_after(&bytes[start..i]);
                continue;
            }
            b'0'..=b'9' => {
                i = skip_number(bytes, i + 1);
                can_start_regex = false;
                continue;
            }
            b'(' | b'[' | b'{' => {
                delimiters.push(match b {
                    b'(' => b')',
                    b'[' => b']',
                    _ => b'}',
                });
                can_start_regex = true;
            }
            b')' | b']' => {
                delimiters_balanced &= delimiters.pop() == Some(b);
                can_start_regex = false;
            }
            b'}' if template_interpolation_depths.last() == Some(&delimiters.len()) => {
                delimiters_balanced &= delimiters.pop() == Some(b'}');
                template_interpolation_depths.pop();
                let (next, has_interpolation) = skip_template_text(bytes, i + 1);
                i = next;
                if has_interpolation {
                    delimiters.push(b'}');
                    template_interpolation_depths.push(delimiters.len());
                    let effective_angle_depth = if track_type_angles { angle_depth } else { 0 };
                    max_depth =
                        max_depth.max(delimiters.len() + effective_angle_depth + decorator_depth);
                    can_start_regex = true;
                } else {
                    can_start_regex = false;
                }
                continue;
            }
            b'}' => {
                delimiters_balanced &= delimiters.pop() == Some(b'}');
                can_start_regex = false;
            }
            b'<' => {
                angle_depth += 1;
                if is_structural_type_angle_open(bytes, i) {
                    structural_type_angle_opens += 1;
                    track_type_angles = structural_type_angle_opens >= 2;
                }
                can_start_regex = true;
            }
            b'>' => {
                angle_depth = angle_depth.saturating_sub(1);
                can_start_regex = true;
            }
            b'@' => {
                decorator_depth += 1;
                can_start_regex = true;
            }
            b'.' => can_start_regex = false,
            b'+' | b'-' if bytes.get(i + 1) == Some(&b) => {
                i += 1;
                can_start_regex = false;
            }
            b',' | b';' | b':' | b'?' | b'!' | b'=' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&'
            | b'|' | b'^' | b'~' => can_start_regex = true,
            // Every byte reaching this arm is identifier-like (`\` starts a
            // Unicode identifier escape, `#` a private name, non-ASCII bytes
            // continue multi-byte identifiers) or an invalid control
            // character. None of them can precede a regex literal, and
            // claiming they do lets `skip_regex` swallow arbitrary source —
            // hiding real brackets from the depth guard (#3107).
            _ => can_start_regex = false,
        }

        let effective_angle_depth = if track_type_angles { angle_depth } else { 0 };
        max_depth = max_depth.max(delimiters.len() + effective_angle_depth + decorator_depth);
        i += 1;
    }

    (
        max_depth,
        delimiters_balanced && delimiters.is_empty() && template_interpolation_depths.is_empty(),
    )
}

pub fn expression_nesting_depth(content: &str) -> usize {
    analyze_expression_nesting(content).0
}

/// Returns whether parentheses, brackets, and braces are correctly paired.
pub fn expression_has_balanced_delimiters(content: &str) -> bool {
    analyze_expression_nesting(content).1
}

/// Returns whether an expression can be handed to OXC's recursive parser safely.
pub fn expression_is_safe_to_parse(content: &str) -> bool {
    let (depth, balanced) = analyze_expression_nesting(content);
    balanced && depth <= MAX_EXPRESSION_NESTING_DEPTH
}

/// Returns true if `content` exceeds [`MAX_EXPRESSION_NESTING_DEPTH`].
#[inline]
pub fn expression_exceeds_max_depth(content: &str) -> bool {
    expression_nesting_depth(content) > MAX_EXPRESSION_NESTING_DEPTH
}

fn skip_quoted(bytes: &[u8], mut i: usize, quote: u8) -> usize {
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = i.saturating_add(2);
        } else if bytes[i] == quote {
            return i + 1;
        } else {
            i += 1;
        }
    }
    i
}

/// Skip literal template text, returning the byte after either the closing
/// backtick or an opening `${`. The caller scans interpolation bodies so deeply
/// nested input is guarded without recursive Rust calls.
fn skip_template_text(bytes: &[u8], mut i: usize) -> (usize, bool) {
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i = i.saturating_add(2),
            b'`' => return (i + 1, false),
            b'$' if bytes.get(i + 1) == Some(&b'{') => return (i + 2, true),
            _ => i += 1,
        }
    }
    (i, false)
}

fn is_structural_type_angle_open(bytes: &[u8], i: usize) -> bool {
    let mut next = i + 1;
    while matches!(bytes.get(next), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        next += 1;
    }
    matches!(bytes.get(next), Some(b'{' | b'['))
}

fn skip_line_comment(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() {
        // Line comments end at any ECMAScript line terminator: LF, CR, LS
        // (U+2028), or PS (U+2029). Stopping only at LF let a bare CR hide
        // parsed code from the guard (#3185).
        match bytes[i] {
            b'\n' | b'\r' => break,
            0xe2 if bytes.get(i + 1) == Some(&0x80)
                && matches!(bytes.get(i + 2), Some(&0xa8) | Some(&0xa9)) =>
            {
                break;
            }
            _ => i += 1,
        }
    }
    i
}

fn skip_block_comment(bytes: &[u8], mut i: usize) -> usize {
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return i + 2;
        }
        i += 1;
    }
    bytes.len()
}

fn skip_regex(bytes: &[u8], mut i: usize) -> usize {
    let mut in_character_class = false;
    while i < bytes.len() {
        match bytes[i] {
            // A regex literal cannot span a line terminator, and `\` before one
            // is not a valid escape: the lexer ends the regex at the terminator.
            // Blindly skipping two bytes would swallow the terminator (LF/CR) or
            // its 0xE2 lead byte (LS/PS), hiding the following source from the
            // guard, so bail before consuming it.
            b'\\' => match bytes.get(i + 1) {
                Some(&b'\n' | &b'\r') => return i,
                Some(&0xe2)
                    if bytes.get(i + 2) == Some(&0x80)
                        && matches!(bytes.get(i + 3), Some(&0xa8) | Some(&0xa9)) =>
                {
                    return i;
                }
                _ => i = i.saturating_add(2),
            },
            b'[' => {
                in_character_class = true;
                i += 1;
            }
            b']' => {
                in_character_class = false;
                i += 1;
            }
            b'/' if !in_character_class => return skip_identifier(bytes, i + 1),
            b'\n' | b'\r' => return i,
            // LS/PS terminate an (unterminated) regex literal like LF/CR.
            0xe2 if bytes.get(i + 1) == Some(&0x80)
                && matches!(bytes.get(i + 2), Some(&0xa8) | Some(&0xa9)) =>
            {
                return i;
            }
            _ => i += 1,
        }
    }
    i
}

fn skip_identifier(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len()
        && matches!(bytes[i], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$')
    {
        i += 1;
    }
    i
}

fn skip_number(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len()
        && matches!(bytes[i], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'.')
    {
        i += 1;
    }
    i
}

fn keyword_allows_regex_after(identifier: &[u8]) -> bool {
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
