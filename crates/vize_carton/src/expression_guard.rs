//! Expression nesting guard: whether text is safe to hand to OXC's recursive parser.
//!
//! Lives in `vize_carton` so armature, transform, and codegen entry points all
//! share one guard through the existing re-export shim.

mod operators;
pub mod scan;

use scan::{
    SpeculativeTypeAngleOpen, keyword_allows_regex_after, skip_block_comment, skip_identifier,
    skip_line_comment, skip_number, skip_quoted, skip_regex, skip_template_text,
    speculative_type_angle_open_kind,
};

pub use scan::is_expression_trailing_trivia;

/// Maximum expression nesting depth accepted before parsing.
///
/// OXC recurses for nested brackets; stack overflow (#956) and a parser timeout
/// at depth 32 (#2944) cannot be caught, so every entry point shares this guard.
pub const MAX_EXPRESSION_NESTING_DEPTH: usize = 31;

/// Per-operator branch depth counted by the cumulative speculative type-angle budget.
/// Logical/nullish operators stop one parser branch, so low-depth comparison chains stay accepted;
/// repeated failed medium-depth type-argument attempts still count toward #4618.
const CUMULATIVE_SPECULATIVE_TYPE_ANGLE_MIN_DEPTH: usize = MAX_EXPRESSION_NESTING_DEPTH / 2;
const MAX_NUMERIC_TOKEN_BYTES: usize = 4096;

struct ExpressionNestingAnalysis {
    max_depth: usize,
    delimiters_balanced: bool,
    cumulative_speculative_type_angle_depth: usize,
    oversized_numeric_token: bool,
}

fn flush_speculative_type_angle_segment(cumulative: &mut usize, segment_depth: &mut usize) {
    if *segment_depth >= CUMULATIVE_SPECULATIVE_TYPE_ANGLE_MIN_DEPTH {
        *cumulative += *segment_depth;
    }
    *segment_depth = 0;
}

/// Returns the maximum parser-recursion depth in `content`.
///
/// Brackets and unambiguous TypeScript angles are paired, while decorator markers
/// accumulate for OXC's recursive parser. Strings, template text, comments, and
/// regexes are skipped; `${...}` template interpolations are scanned.
fn analyze_expression_nesting(content: &str) -> ExpressionNestingAnalysis {
    let bytes = content.as_bytes();
    let (mut angle_depth, mut decorator_depth) = (0usize, 0usize);
    let mut max_depth = 0usize;
    let mut delimiters = Vec::new();
    let mut delimiters_balanced = true;
    let mut can_start_regex = true;
    let mut template_interpolation_depths = Vec::new();
    // Plain `foo < bar` is indistinguishable from a type argument to a byte
    // scanner. Enter angle-tracking mode only for repeated speculative type
    // prefixes: the structural shapes (`<{`, `<[`, #2944), JSDoc non-nullable
    // shape (`<!`, #3213), parenthesized-type shape (`<(`, #3277/#3279/#3281),
    // and identifier-led type references (`<T`, #3712). OXC's type speculation
    // recurses once per marker and nested `<` until the Rust stack overflows or
    // the rewind cascade goes super-linear. All are discovered while scanning so
    // strings and comments cannot activate the mode.
    //
    // Only *unclosed* angles accumulate, and logical/nullish operators (`&&`,
    // `||`, `??`) cannot appear inside a type-argument list, so they reset this
    // speculation: a flat boolean chain of `<` comparisons is not a type run.
    // Those two escapes are what keep the identifier arm — by far the broadest
    // of the four — off ordinary code: every `>` pays an angle back, and any
    // real boolean chain resets. Measured against the #3712 reproducer, both
    // escapes also match OXC: injecting `&&` every 25 `<`, or closing the
    // angles, drops a 10.7KB input from 1.47s to ~25us.
    let mut speculative_type_angle_opens = 0usize;
    let mut malformed_type_escape_opens = 0usize;
    let mut segment_speculative_type_angle_depth = 0usize;
    let mut cumulative_speculative_type_angle_depth = 0usize;
    let mut oversized_numeric_token = false;
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
                    let malformed_escape_depth = if track_type_angles {
                        malformed_type_escape_opens
                    } else {
                        0
                    };
                    max_depth = max_depth.max(
                        delimiters.len()
                            + effective_angle_depth
                            + malformed_escape_depth
                            + decorator_depth,
                    );
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
            // A regex literal that never closes is not one: the lexer reports an
            // unterminated regex and recovers, so the bytes stay live for the
            // parser. Skipping them anyway hid 183 unclosed type angles behind a
            // single `/` running to EOF — the guard scored that input at depth 6
            // while OXC speculated over every angle until it ran out of memory
            // (#3873) — and 6182 more behind a `/` that a line terminator closed
            // 27 KiB later (#3875). Falling through scans them as ordinary
            // source instead, which can only over-count.
            b'/' if can_start_regex => {
                if let Some(next) = skip_regex(bytes, i + 1) {
                    i = next;
                    can_start_regex = false;
                    continue;
                }
                can_start_regex = true;
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                let start = i;
                i = skip_identifier(bytes, i + 1);
                can_start_regex = keyword_allows_regex_after(&bytes[start..i]);
                continue;
            }
            b'0'..=b'9' => {
                let start = i;
                i = skip_number(bytes, i + 1);
                oversized_numeric_token |= i - start > MAX_NUMERIC_TOKEN_BYTES;
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
                    let malformed_escape_depth = if track_type_angles {
                        malformed_type_escape_opens
                    } else {
                        0
                    };
                    max_depth = max_depth.max(
                        delimiters.len()
                            + effective_angle_depth
                            + malformed_escape_depth
                            + decorator_depth,
                    );
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
                if let Some(kind) = speculative_type_angle_open_kind(content, i) {
                    speculative_type_angle_opens += 1;
                    if kind == SpeculativeTypeAngleOpen::MalformedIdentifierEscape {
                        malformed_type_escape_opens += 1;
                    }
                    track_type_angles = speculative_type_angle_opens >= 2;
                }
                can_start_regex = true;
            }
            // A `>` that pays back an open angle closes a type-argument list, and
            // a `/` after one is division — OXC never starts a regex there. The
            // arm used to allow a regex after every `>`, so `Props<{ … }>/(((…`
            // handed `skip_regex` the rest of the line and hid its bracket run
            // from the depth budget while OXC kept every paren live and recursed
            // to a stack overflow (#3858). Only a relational `>`, with no angle
            // outstanding, can precede a regex (`a > /re/.test(b)`).
            b'>' => {
                let closed_type_angle = angle_depth > 0;
                angle_depth = angle_depth.saturating_sub(1);
                if angle_depth == 0 {
                    speculative_type_angle_opens = 0;
                    malformed_type_escape_opens = 0;
                    track_type_angles = false;
                }
                can_start_regex = !closed_type_angle;
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
                flush_speculative_type_angle_segment(
                    &mut cumulative_speculative_type_angle_depth,
                    &mut segment_speculative_type_angle_depth,
                );
                speculative_type_angle_opens = 0;
                malformed_type_escape_opens = 0;
                track_type_angles = false;
                angle_depth = 0;
                i += 1;
                can_start_regex = true;
            }
            b'?' if bytes.get(i + 1) == Some(&b'?') => {
                flush_speculative_type_angle_segment(
                    &mut cumulative_speculative_type_angle_depth,
                    &mut segment_speculative_type_angle_depth,
                );
                speculative_type_angle_opens = 0;
                malformed_type_escape_opens = 0;
                track_type_angles = false;
                angle_depth = 0;
                i += 1;
                can_start_regex = true;
            }
            b',' | b';' | b':' | b'?' | b'!' | b'=' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&'
            | b'|' | b'^' | b'~' => can_start_regex = true,
            // A `\` in code position begins an identifier escape for OXC's
            // lexer (`\uXXXX` / `\u{...}`); invalid escapes recover without opening
            // a new literal. Either way OXC never starts a string at a `'`/`"` — or a
            // template at a `` ` `` — that immediately follows a `\`. The
            // scanner used to advance past the lone `\` and then treat that
            // quote as a string opener, so `skip_quoted` ran to the next
            // unescaped quote (or EOF), swallowing the bracket and type-angle
            // runs that drive OXC's exponential type-argument speculation and
            // hiding them from the depth guard (#3271). The same held for a
            // backtick: `skip_template_text` swallowed the bracket run behind a
            // phantom template literal while OXC kept every bracket live and
            // recursed to a stack overflow (#3274). Consuming the neutralized
            // quote or backtick with the backslash keeps that following source
            // visible to the depth budget. A `\` before any other byte falls
            // through to the normal arms, so `\(` / `\[` / `\{` still count
            // their brackets exactly as OXC keeps them live.
            b'\\' => {
                if matches!(bytes.get(i + 1), Some(b'\'' | b'"' | b'`')) {
                    i += 1;
                }
                can_start_regex = false;
            }
            // Identifier-like (`#`, non-ASCII) or invalid bytes cannot precede
            // regex; doing so hid real brackets behind a false regex (#3107).
            _ => can_start_regex = false,
        }

        let effective_angle_depth = if track_type_angles { angle_depth } else { 0 };
        let malformed_escape_depth = if track_type_angles {
            malformed_type_escape_opens
        } else {
            0
        };
        if track_type_angles {
            segment_speculative_type_angle_depth =
                segment_speculative_type_angle_depth.max(angle_depth + malformed_escape_depth);
        }
        max_depth = max_depth.max(
            delimiters.len() + effective_angle_depth + malformed_escape_depth + decorator_depth,
        );
        i += 1;
    }

    flush_speculative_type_angle_segment(
        &mut cumulative_speculative_type_angle_depth,
        &mut segment_speculative_type_angle_depth,
    );

    ExpressionNestingAnalysis {
        max_depth,
        delimiters_balanced: delimiters_balanced
            && delimiters.is_empty()
            && template_interpolation_depths.is_empty(),
        cumulative_speculative_type_angle_depth,
        oversized_numeric_token,
    }
}

pub fn expression_nesting_depth(content: &str) -> usize {
    analyze_expression_nesting(content).max_depth
}

/// Returns whether parentheses, brackets, and braces are correctly paired.
pub fn expression_has_balanced_delimiters(content: &str) -> bool {
    analyze_expression_nesting(content).delimiters_balanced
}

/// Returns whether an expression can be handed to OXC's recursive parser safely.
pub fn expression_is_safe_to_parse(content: &str) -> bool {
    let analysis = analyze_expression_nesting(content);
    analysis.delimiters_balanced
        && analysis.max_depth <= MAX_EXPRESSION_NESTING_DEPTH
        && analysis.cumulative_speculative_type_angle_depth <= MAX_EXPRESSION_NESTING_DEPTH
        && !analysis.oversized_numeric_token
        && !operators::has_excessive_prefix_operator_run(content)
}

/// Returns true if `content` exceeds [`MAX_EXPRESSION_NESTING_DEPTH`].
#[inline]
pub fn expression_exceeds_max_depth(content: &str) -> bool {
    expression_nesting_depth(content) > MAX_EXPRESSION_NESTING_DEPTH
}
