#[path = "nesting/scan.rs"]
mod scan;

use scan::{
    is_speculative_type_angle_open, keyword_allows_regex_after, skip_block_comment,
    skip_identifier, skip_line_comment, skip_number, skip_quoted, skip_regex, skip_template_text,
};

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
    // scanner. Enter angle-tracking mode only for repeated speculative type
    // prefixes: the structural shapes of the parser-timeout class (`<{`, `<[`,
    // #2944) and the JSDoc non-nullable shape (`<!`, #3213), where OXC's type
    // speculation recurses once per `!` and per nested `<` until the Rust
    // stack overflows. Both are discovered while scanning so strings and
    // comments cannot activate the mode. Logical/nullish operators (`&&`, `||`,
    // `??`) cannot appear inside a type-argument list, so they reset this
    // speculation: a flat boolean chain of `<` comparisons is not a type run.
    let mut speculative_type_angle_opens = 0usize;
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
                if is_speculative_type_angle_open(content, i) {
                    speculative_type_angle_opens += 1;
                    track_type_angles = speculative_type_angle_opens >= 2;
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
            // Logical AND/OR and nullish coalescing cannot appear inside a
            // type-argument list, so they end the current candidate type chain.
            // Without this reset a later `<` comparison in the same flat boolean
            // expression keeps accumulating `angle_depth`, tripping the depth
            // guard on a valid chain past the limit (#3213 follow-up). A single
            // `&`/`|`/`?` (bitwise, union/intersection type, optional/ternary)
            // stays inside the speculation and is handled below.
            b'&' | b'|' if bytes.get(i + 1) == Some(&b) => {
                speculative_type_angle_opens = 0;
                track_type_angles = false;
                angle_depth = 0;
                i += 1;
                can_start_regex = true;
            }
            b'?' if bytes.get(i + 1) == Some(&b'?') => {
                speculative_type_angle_opens = 0;
                track_type_angles = false;
                angle_depth = 0;
                i += 1;
                can_start_regex = true;
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
