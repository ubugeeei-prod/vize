use memchr::memchr;
use std::borrow::Cow;
use vize_carton::{cstr, FxHashMap, String};

// Static closing tag prefixes without '>' for fast comparison (avoid format!)
const CLOSING_SCRIPT: &[u8] = b"</script";
const CLOSING_STYLE: &[u8] = b"</style";

// Tag name bytes for fast comparison
const TAG_TEMPLATE: &[u8] = b"template";
const TAG_SCRIPT: &[u8] = b"script";
const TAG_STYLE: &[u8] = b"style";

/// Build a uniform `(code, message)` error for any malformed block.
fn build_malformed_error(tag_name: &[u8], reason: &str) -> (&'static str, String) {
    let tag_str = std::str::from_utf8(tag_name).unwrap_or("unknown");
    (
        "MALFORMED_BLOCK",
        cstr!("Malformed <{tag_str}> block: {reason}."),
    )
}

/// Fast tag name comparison using byte slices
#[inline(always)]
pub(super) fn tag_name_eq(name: &[u8], expected: &[u8]) -> bool {
    name.len() == expected.len() && name.eq_ignore_ascii_case(expected)
}

/// Fast byte slice prefix check
#[inline(always)]
fn starts_with_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.len() >= needle.len() && haystack[..needle.len()].eq_ignore_ascii_case(needle)
}

/// Fast tag name character check
#[inline(always)]
fn is_tag_name_char_fast(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_')
}

/// Fast whitespace check
#[inline(always)]
fn is_whitespace_fast(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

/// Parse a single block from the source using byte operations
/// Returns borrowed strings using Cow for zero-copy
///
/// - `Ok(Some(...))` — successfully parsed block.
/// - `Ok(None)`      — can't find a block at this position (stray `<`, comment, etc.)
/// - `Err((code, message))` — position starts an invalid/incomplete block
pub(super) fn parse_block_fast<'a>(
    bytes: &[u8],
    source: &'a str,
    start: usize,
    start_line: usize,
) -> Result<
    Option<(
        &'a [u8],                              // tag name as bytes
        FxHashMap<Cow<'a, str>, Cow<'a, str>>, // attrs with borrowed strings
        Cow<'a, str>,                          // content as borrowed string
        usize,                                 // content start
        usize,                                 // content end
        usize,                                 // end position
        usize,                                 // end line
        usize,                                 // end column
    )>,
    (&'static str, String), // (error_code, error_message)
> {
    let len = bytes.len();

    // Skip '<'
    let mut pos = start + 1;
    if pos >= len {
        return Ok(None);
    }

    // Parse tag name - find end of tag name
    let tag_start = pos;
    while pos < len && is_tag_name_char_fast(bytes[pos]) {
        pos += 1;
    }

    if pos == tag_start {
        return Ok(None);
    }

    let tag_name = &source.as_bytes()[tag_start..pos];

    // Parse attributes with zero-copy
    let mut attrs: FxHashMap<Cow<'a, str>, Cow<'a, str>> = FxHashMap::default();

    while pos < len && bytes[pos] != b'>' {
        // Skip whitespace
        while pos < len && is_whitespace_fast(bytes[pos]) {
            pos += 1;
        }

        if pos >= len || bytes[pos] == b'>' || bytes[pos] == b'/' {
            break;
        }

        // Parse attribute name
        let attr_start = pos;
        while pos < len {
            let c = bytes[pos];
            if c == b'='
                || c == b' '
                || c == b'>'
                || c == b'/'
                || c == b'\t'
                || c == b'\n'
                || c == b'\r'
            {
                break;
            }
            pos += 1;
        }

        if pos == attr_start {
            pos += 1;
            continue;
        }

        // Zero-copy: borrow from source
        let attr_name: Cow<'a, str> = Cow::Borrowed(&source[attr_start..pos]);

        // Skip whitespace
        while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
            pos += 1;
        }

        let attr_value: Cow<'a, str> = if pos < len && bytes[pos] == b'=' {
            pos += 1;

            // Skip whitespace
            while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
                pos += 1;
            }

            if pos < len && (bytes[pos] == b'"' || bytes[pos] == b'\'') {
                let quote_char = bytes[pos];
                pos += 1;
                let value_start = pos;

                // Use memchr for fast quote finding
                if let Some(quote_pos) = memchr(quote_char, &bytes[pos..]) {
                    pos += quote_pos;
                    let value = Cow::Borrowed(&source[value_start..pos]);
                    pos += 1; // Skip closing quote
                    value
                } else {
                    // No closing quote found
                    while pos < len && bytes[pos] != quote_char {
                        pos += 1;
                    }
                    let value = Cow::Borrowed(&source[value_start..pos]);
                    if pos < len {
                        pos += 1;
                    }
                    value
                }
            } else {
                // Unquoted value
                let value_start = pos;
                while pos < len {
                    let c = bytes[pos];
                    if c == b' ' || c == b'>' || c == b'/' || c == b'\t' || c == b'\n' {
                        break;
                    }
                    pos += 1;
                }
                Cow::Borrowed(&source[value_start..pos])
            }
        } else {
            // Boolean attribute
            Cow::Borrowed("")
        };

        if !attr_name.is_empty() {
            attrs.insert(attr_name, attr_value);
        }
    }

    // Handle self-closing tag
    let is_self_closing = pos > 0 && pos < len && bytes[pos - 1] == b'/';

    if is_self_closing {
        if pos < len && bytes[pos] == b'>' {
            pos += 1;
        }
        return Ok(Some((
            tag_name,
            attrs,
            Cow::Borrowed(""),
            pos,
            pos,
            pos,
            start_line,
            pos - start,
        )));
    }

    // Skip '>'
    if pos < len && bytes[pos] == b'>' {
        pos += 1;
    } else {
        // Opening tag is not closed — error for any tag.
        return Err(build_malformed_error(
            tag_name,
            "the opening tag is incomplete",
        ));
    }

    let content_start = pos;

    // Find closing tag based on tag type
    let mut line = start_line;
    let mut last_newline = start;

    // Handle known tags with static closing tags
    if tag_name.eq_ignore_ascii_case(TAG_TEMPLATE) {
        // Template block: handle nested template tags
        let mut depth = 1;

        // Check for closing template tag, handling whitespace before the closing '>'
        // This handles cases like:
        //   </template>       - normal
        //   </template\n   >  - closing '>' on next line
        fn is_closing_template_tag(bytes: &[u8], pos: usize, len: usize) -> Option<usize> {
            // Check if we have "</template" (without the final ">")
            const CLOSING_TAG_PREFIX: &[u8] = b"</template";
            if pos + CLOSING_TAG_PREFIX.len() > len {
                return None;
            }
            if !bytes[pos..pos + CLOSING_TAG_PREFIX.len()].eq_ignore_ascii_case(CLOSING_TAG_PREFIX)
            {
                return None;
            }
            // Find the closing '>' allowing whitespace
            let mut check_pos = pos + CLOSING_TAG_PREFIX.len();
            while check_pos < len {
                match bytes[check_pos] {
                    b'>' => return Some(check_pos + 1), // Return position after '>'
                    b' ' | b'\t' | b'\n' | b'\r' => check_pos += 1,
                    _ => return None, // Invalid character in closing tag
                }
            }
            None
        }

        while pos < len {
            if bytes[pos] == b'\n' {
                line += 1;
                last_newline = pos;
            }

            if bytes[pos] == b'<' {
                // Check for closing tag using byte comparison
                if let Some(end_tag_pos) = is_closing_template_tag(bytes, pos, len) {
                    depth -= 1;
                    if depth == 0 {
                        let content_end = pos;
                        let end_pos = end_tag_pos;
                        let col = pos - last_newline + (end_pos - pos);
                        let content = Cow::Borrowed(&source[content_start..content_end]);
                        return Ok(Some((
                            tag_name,
                            attrs,
                            content,
                            content_start,
                            content_end,
                            end_pos,
                            line,
                            col,
                        )));
                    }
                    pos = end_tag_pos;
                    continue;
                }

                // Check for nested opening tag
                if starts_with_bytes(&bytes[pos + 1..], TAG_TEMPLATE) {
                    let tag_check_pos = pos + 1 + TAG_TEMPLATE.len();
                    if tag_check_pos < len {
                        let next_char = bytes[tag_check_pos];
                        if next_char == b' '
                            || next_char == b'>'
                            || next_char == b'\n'
                            || next_char == b'\t'
                            || next_char == b'\r'
                        {
                            // Check if self-closing
                            let mut check_pos = tag_check_pos;
                            let mut is_self_closing_nested = false;
                            while check_pos < len && bytes[check_pos] != b'>' {
                                if bytes[check_pos] == b'/'
                                    && check_pos + 1 < len
                                    && bytes[check_pos + 1] == b'>'
                                {
                                    is_self_closing_nested = true;
                                    break;
                                }
                                check_pos += 1;
                            }
                            if !is_self_closing_nested {
                                depth += 1;
                            }
                        }
                    }
                }
            }

            pos += 1;
        }
        // Exhausted input without finding </template>
        return Err(build_malformed_error(
            tag_name,
            "the closing tag is missing",
        ));
    }

    // Script/style blocks: use static closing tags
    let closing_tag = if tag_name.eq_ignore_ascii_case(TAG_SCRIPT) {
        CLOSING_SCRIPT
    } else if tag_name.eq_ignore_ascii_case(TAG_STYLE) {
        CLOSING_STYLE
    } else {
        // Custom block: need to find closing tag dynamically
        return find_custom_block_end(
            bytes,
            source,
            tag_name,
            pos,
            content_start,
            start_line,
            attrs,
        );
    };

    // For script blocks, we need to be aware of string literals to avoid
    // matching closing tags inside strings like: const x = `</script>`
    let is_script = tag_name.eq_ignore_ascii_case(TAG_SCRIPT);

    // Track the previous non-whitespace character to determine string context
    let mut prev_significant_char: u8 = b'\n'; // Start as if at beginning of line

    while pos < len {
        let b = bytes[pos];

        if b == b'\n' {
            line += 1;
            last_newline = pos;
            prev_significant_char = b'\n';
            pos += 1;
            continue;
        }

        // Skip whitespace but don't update prev_significant_char
        if b == b' ' || b == b'\t' || b == b'\r' {
            pos += 1;
            continue;
        }

        // For script blocks, skip over comments and string literals
        if is_script {
            // Check for single-line comment
            if b == b'/' && pos + 1 < len && bytes[pos + 1] == b'/' {
                // Skip to end of line
                pos += 2;
                while pos < len && bytes[pos] != b'\n' {
                    pos += 1;
                }
                continue;
            }

            // Check for multi-line comment
            if b == b'/' && pos + 1 < len && bytes[pos + 1] == b'*' {
                pos += 2;
                while pos + 1 < len {
                    if bytes[pos] == b'\n' {
                        line += 1;
                        last_newline = pos;
                    }
                    if bytes[pos] == b'*' && bytes[pos + 1] == b'/' {
                        pos += 2;
                        break;
                    }
                    pos += 1;
                }
                continue;
            }

            // Check for string literals (', ", `)
            // Only treat as string if in a context where strings are expected
            // (after =, (, [, ,, :, {, or at start of expression)
            // This avoids treating quotes inside regex literals as strings
            //
            // For backticks specifically, also allow after alphanumeric characters
            // to handle tagged templates (e.g., html`...`) and keywords (e.g., return `...`)
            if b == b'\'' || b == b'"' || b == b'`' {
                let is_string_context = matches!(
                    prev_significant_char,
                    b'=' | b'('
                        | b'['
                        | b','
                        | b':'
                        | b'{'
                        | b';'
                        | b'\n'
                        | b'?'
                        | b'&'
                        | b'|'
                        | b'+'
                        | b'-'
                        | b'*'
                        | b'!'
                        | b'>'
                        | b'<'
                        | b'%'
                        | b'^'
                ) || (b == b'`'
                    && (prev_significant_char.is_ascii_alphanumeric()
                        || prev_significant_char == b'_'
                        || prev_significant_char == b')'));

                if is_string_context {
                    let quote = b;
                    pos += 1;

                    while pos < len {
                        let c = bytes[pos];

                        if c == b'\n' {
                            line += 1;
                            last_newline = pos;
                        }

                        // Handle escape sequences
                        if c == b'\\' && pos + 1 < len {
                            pos += 2; // Skip escaped character
                            continue;
                        }

                        // Handle template literal expressions ${...}
                        if quote == b'`' && c == b'$' && pos + 1 < len && bytes[pos + 1] == b'{' {
                            pos += 2;
                            let mut brace_depth = 1;
                            while pos < len && brace_depth > 0 {
                                let inner = bytes[pos];
                                if inner == b'\n' {
                                    line += 1;
                                    last_newline = pos;
                                }
                                if inner == b'{' {
                                    brace_depth += 1;
                                } else if inner == b'}' {
                                    brace_depth -= 1;
                                } else if inner == b'\\' && pos + 1 < len {
                                    pos += 1; // Skip escape in template expression
                                }
                                pos += 1;
                            }
                            continue;
                        }

                        // End of string
                        if c == quote {
                            pos += 1;
                            break;
                        }

                        // For non-template strings, newline ends the string (syntax error, but handle gracefully)
                        if quote != b'`' && c == b'\n' {
                            break;
                        }

                        pos += 1;
                    }
                    prev_significant_char = quote; // String ended with quote
                    continue;
                }
            }
        }

        // Check for closing tag
        if b == b'<' && starts_with_bytes(&bytes[pos..], closing_tag) {
            let after = pos + closing_tag.len();
            if after < len && bytes[after] == b'>' {
                let content_end = pos;
                let end_pos = after + 1;
                let col = pos - last_newline + closing_tag.len() + 1;
                let content = Cow::Borrowed(&source[content_start..content_end]);
                return Ok(Some((
                    tag_name,
                    attrs,
                    content,
                    content_start,
                    content_end,
                    end_pos,
                    line,
                    col,
                )));
            }
            return Err(build_malformed_error(
                tag_name,
                "the closing tag is incomplete",
            ));
        }

        prev_significant_char = b;
        pos += 1;
    }

    Err(build_malformed_error(
        tag_name,
        "the closing tag is missing",
    ))
}

/// Find the end of a custom block (non-template/script/style)
fn find_custom_block_end<'a>(
    bytes: &[u8],
    source: &'a str,
    tag_name: &'a [u8],
    mut pos: usize,
    content_start: usize,
    start_line: usize,
    attrs: FxHashMap<Cow<'a, str>, Cow<'a, str>>,
) -> Result<
    Option<(
        &'a [u8],
        FxHashMap<Cow<'a, str>, Cow<'a, str>>,
        Cow<'a, str>,
        usize,
        usize,
        usize,
        usize,
        usize,
    )>,
    (&'static str, String),
> {
    let len = bytes.len();
    let mut line = start_line;
    let mut last_newline = content_start;

    while pos < len {
        if let Some(lt_offset) = memchr(b'<', &bytes[pos..]) {
            // Count newlines
            for &b in &bytes[pos..pos + lt_offset] {
                if b == b'\n' {
                    line += 1;
                    last_newline = pos + lt_offset;
                }
            }
            pos += lt_offset;

            // Check for </
            if pos + 2 < len && bytes[pos] == b'<' && bytes[pos + 1] == b'/' {
                let close_tag_start = pos + 2;
                // Check if tag name matches
                if close_tag_start + tag_name.len() <= len
                    && bytes[close_tag_start..close_tag_start + tag_name.len()]
                        .eq_ignore_ascii_case(tag_name)
                {
                    // Check for closing >
                    let after_name = close_tag_start + tag_name.len();
                    if after_name < len && bytes[after_name] == b'>' {
                        let content_end = pos;
                        let end_pos = after_name + 1;
                        let col = pos - last_newline + (end_pos - pos);
                        let content = Cow::Borrowed(&source[content_start..content_end]);
                        return Ok(Some((
                            tag_name,
                            attrs,
                            content,
                            content_start,
                            content_end,
                            end_pos,
                            line,
                            col,
                        )));
                    }
                }
            }
            pos += 1;
        } else {
            break;
        }
    }

    // Exhausted input without finding the closing tag.
    Err(build_malformed_error(
        tag_name,
        "the closing tag is missing",
    ))
}
