use memchr::{memchr, memchr_iter, memmem};
use std::borrow::Cow;
use vize_carton::{cstr, FxHashMap, String};

// Tag name bytes for fast comparison
const TAG_TEMPLATE: &[u8] = b"template";
const TAG_SCRIPT: &[u8] = b"script";
const TAG_STYLE: &[u8] = b"style";

type BlockAttrs<'a> = FxHashMap<Cow<'a, str>, Cow<'a, str>>;
type BlockParseOutput<'a> = (
    &'a [u8],       // tag name as bytes
    BlockAttrs<'a>, // attrs with borrowed strings
    Cow<'a, str>,   // content as borrowed string
    usize,          // content start
    usize,          // content end
    usize,          // end position
    usize,          // end line
    usize,          // end column
);
type BlockParseError = (&'static str, String);
type BlockParseResult<'a> = Result<Option<BlockParseOutput<'a>>, BlockParseError>;

struct BlockEndSearch<'a> {
    bytes: &'a [u8],
    source: &'a str,
    tag_name: &'a [u8],
    pos: usize,
    content_start: usize,
    start_line: usize,
    initial_last_newline: usize,
    attrs: BlockAttrs<'a>,
}

/// Build a uniform `(code, message)` error for any malformed block.
fn build_malformed_error(tag_name: &[u8], reason: &str) -> BlockParseError {
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

#[inline]
fn advance_line(bytes: &[u8], base: usize, line: &mut usize, last_newline: &mut usize) {
    for offset in memchr_iter(b'\n', bytes) {
        *line += 1;
        *last_newline = base + offset;
    }
}

/// Find the end of a closing tag `</tag_name` followed by optional whitespace and `>`.
/// Returns the position immediately after `>`, or `None` if no valid closing tag at `pos`.
#[inline]
fn find_closing_tag_end(bytes: &[u8], pos: usize, len: usize, tag_name: &[u8]) -> Option<usize> {
    // Need at least "</" + tag_name + ">"
    if pos + 2 + tag_name.len() >= len {
        return None;
    }
    if bytes[pos] != b'<' || bytes[pos + 1] != b'/' {
        return None;
    }
    let name_start = pos + 2;
    if !bytes[name_start..name_start + tag_name.len()].eq_ignore_ascii_case(tag_name) {
        return None;
    }
    let mut check_pos = name_start + tag_name.len();
    while check_pos < len {
        match bytes[check_pos] {
            b'>' => return Some(check_pos + 1),
            b' ' | b'\t' | b'\n' | b'\r' => check_pos += 1,
            _ => return None,
        }
    }
    None
}

/// Parse a single block from the source using byte operations
/// Returns borrowed strings using Cow for zero-copy
///
/// - `Ok(Some(...))` — successfully parsed block.
/// - `Ok(None)` — no SFC block starts at this position.
/// - `Err(...)` — a block starts here but is incomplete or malformed.
pub(super) fn parse_block_fast<'a>(
    bytes: &'a [u8],
    source: &'a str,
    start: usize,
    start_line: usize,
) -> BlockParseResult<'a> {
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
    let mut attrs: BlockAttrs<'a> = FxHashMap::default();

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
            let Some(lt_offset) = memchr(b'<', &bytes[pos..]) else {
                advance_line(&bytes[pos..], pos, &mut line, &mut last_newline);
                break;
            };

            advance_line(
                &bytes[pos..pos + lt_offset],
                pos,
                &mut line,
                &mut last_newline,
            );
            pos += lt_offset;

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

            pos += 1;
        }
        return Err(build_malformed_error(
            tag_name,
            "the closing tag is missing",
        ));
    }

    if tag_name.eq_ignore_ascii_case(TAG_STYLE) {
        return find_block_end(BlockEndSearch {
            bytes,
            source,
            tag_name,
            pos,
            content_start,
            start_line,
            initial_last_newline: start,
            attrs,
        });
    }

    // Custom block: need to find closing tag dynamically
    if !tag_name.eq_ignore_ascii_case(TAG_SCRIPT) {
        return find_block_end(BlockEndSearch {
            bytes,
            source,
            tag_name,
            pos,
            content_start,
            start_line,
            initial_last_newline: content_start,
            attrs,
        });
    }

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
                if let Some(newline_offset) = memchr(b'\n', &bytes[pos..]) {
                    pos += newline_offset;
                } else {
                    pos = len;
                }
                continue;
            }

            // Check for multi-line comment
            if b == b'/' && pos + 1 < len && bytes[pos + 1] == b'*' {
                pos += 2;
                if let Some(end_offset) = memmem::find(&bytes[pos..], b"*/") {
                    advance_line(
                        &bytes[pos..pos + end_offset],
                        pos,
                        &mut line,
                        &mut last_newline,
                    );
                    pos += end_offset + 2;
                } else {
                    advance_line(&bytes[pos..], pos, &mut line, &mut last_newline);
                    pos = len;
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

        // Check for closing tag (allows optional whitespace before '>')
        if b == b'<' {
            if let Some(end_tag_pos) = find_closing_tag_end(bytes, pos, len, tag_name) {
                let content_end = pos;
                let col = pos - last_newline + (end_tag_pos - pos);
                let content = Cow::Borrowed(&source[content_start..content_end]);
                return Ok(Some((
                    tag_name,
                    attrs,
                    content,
                    content_start,
                    content_end,
                    end_tag_pos,
                    line,
                    col,
                )));
            }
        }

        prev_significant_char = b;
        pos += 1;
    }

    Err(build_malformed_error(
        tag_name,
        "the closing tag is missing",
    ))
}

/// Find the end of a raw block by jumping between `<` bytes.
fn find_block_end<'a>(search: BlockEndSearch<'a>) -> BlockParseResult<'a> {
    let BlockEndSearch {
        bytes,
        source,
        tag_name,
        mut pos,
        content_start,
        start_line,
        initial_last_newline,
        attrs,
    } = search;
    let len = bytes.len();
    let mut line = start_line;
    let mut last_newline = initial_last_newline;

    while pos < len {
        if let Some(lt_offset) = memchr(b'<', &bytes[pos..]) {
            // Count newlines
            advance_line(
                &bytes[pos..pos + lt_offset],
                pos,
                &mut line,
                &mut last_newline,
            );
            pos += lt_offset;

            // Check for closing tag (allows optional whitespace before '>')
            if bytes[pos] == b'<' {
                if let Some(end_tag_pos) = find_closing_tag_end(bytes, pos, len, tag_name) {
                    let content_end = pos;
                    let col = pos - last_newline + (end_tag_pos - pos);
                    let content = Cow::Borrowed(&source[content_start..content_end]);
                    return Ok(Some((
                        tag_name,
                        attrs,
                        content,
                        content_start,
                        content_end,
                        end_tag_pos,
                        line,
                        col,
                    )));
                }
            }
            pos += 1;
        } else {
            break;
        }
    }

    Err(build_malformed_error(
        tag_name,
        "the closing tag is missing",
    ))
}
