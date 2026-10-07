use super::{
    ExpressionNestingAnalysis, MAX_EXPRESSION_NESTING_DEPTH, MAX_NUMERIC_TOKEN_BYTES,
    SpeculativeTypeAngleOpen, arrow_default_angle_product_exceeds,
    flush_speculative_type_angle_segment, keyword_allows_regex_after, skip_block_comment,
    skip_identifier, skip_line_comment, skip_number, skip_quoted, skip_regex, skip_template_text,
    speculative_arrow_default_paren, speculative_type_angle_open_kind,
    starts_valid_identifier_escape,
};

/// `<(ident =` only follows `<`, trivia, or a comment. Other next bytes skip the lookahead.
fn arrow_default_paren_possible(bytes: &[u8], open: usize) -> bool {
    match bytes.get(open + 1) {
        Some(b'(' | b' ' | b'\t' | 0x0b | 0x0c | b'\n' | b'\r' | b'/') => true,
        Some(byte) if *byte >= 0x80 => true,
        Some(_) | None => false,
    }
}

/// Returns the maximum parser-recursion depth in `content`.
///
/// Brackets and unambiguous TypeScript angles are paired, while decorator markers
/// accumulate for OXC's recursive parser. Strings, template text, comments, and
/// regexes are skipped; `${...}` template interpolations are scanned.
#[inline(always)]
pub(super) fn analyze_expression_nesting(content: &str) -> ExpressionNestingAnalysis {
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
    //
    // `<(ident =` is the exception (#7116): OXC parses it as a function type
    // whose parameter default is an expression, so `||` inside the default does
    // not end the outer speculation. `<` counts on each open default are tracked
    // separately and rejected when their product blows up.
    let mut speculative_type_angle_opens = 0usize;
    // Closing `>` pays back recursion depth, but it does not make an outer
    // speculative type-argument parse succeed. Reopening more candidates in
    // that same unclosed chain can make OXC retry its failed branches many
    // times even when the peak angle depth stays below the depth limit.
    let mut excessive_speculative_type_angle_opens = false;
    let mut malformed_type_escape_opens = 0usize;
    let mut segment_speculative_type_angle_depth = 0usize;
    let mut cumulative_speculative_type_angle_depth = 0usize;
    let mut oversized_numeric_token = false;
    let mut track_type_angles = false;
    // `(delimiter depth at the opener, `<` count)` for each `<(ident =` frame.
    // Ordinary parentheses do not push: the depth tells which `)` closes the frame.
    let mut arrow_frames: Vec<(usize, usize)> = Vec::new();
    let mut pending_arrow_default_paren: Option<usize> = None;
    let mut excessive_arrow_default_speculation = false;
    let mut i = 0;

    while let Some(&b) = bytes.get(i) {
        match b {
            b' ' | b'\t' | b'\r' | b'\n' | 0x0b | 0x0c => {
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
                can_start_regex =
                    keyword_allows_regex_after(bytes.get(start..i).unwrap_or_default());
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
                if b == b'(' {
                    if pending_arrow_default_paren == Some(i) {
                        arrow_frames.push((delimiters.len(), 0));
                    }
                    pending_arrow_default_paren = None;
                }
                delimiters.push(match b {
                    b'(' => b')',
                    b'[' => b']',
                    _ => b'}',
                });
                can_start_regex = true;
            }
            b')' | b']' => {
                if b == b')'
                    && arrow_frames
                        .last()
                        .is_some_and(|(depth, _)| depth.saturating_add(1) == delimiters.len())
                {
                    arrow_frames.pop();
                }
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
                pending_arrow_default_paren = if arrow_default_paren_possible(bytes, i) {
                    speculative_arrow_default_paren(content, i)
                } else {
                    None
                };
                if let Some((_, angles)) = arrow_frames.last_mut() {
                    *angles += 1;
                }
                if !arrow_frames.is_empty() {
                    excessive_arrow_default_speculation |=
                        arrow_default_angle_product_exceeds(&arrow_frames);
                }
                if let Some(kind) = speculative_type_angle_open_kind(content, i) {
                    speculative_type_angle_opens += 1;
                    excessive_speculative_type_angle_opens |=
                        speculative_type_angle_opens > MAX_EXPRESSION_NESTING_DEPTH;
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
            // A postfix non-null assertion makes `value!/=...` division
            // assignment; misreading `/=` as regex hides parser depth (#7275).
            b'!' => can_start_regex |= bytes.get(i + 1) == Some(&b'='),
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
            b',' | b';' | b':' | b'?' | b'=' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|'
            | b'^' | b'~' => can_start_regex = true,
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
            // quote, backtick, or slash with the backslash keeps the following
            // source visible. Backslash pairs leave literal openers live (#7808).
            // OXC consumes the slash of an invalid `\/` escape,
            // so it cannot open a comment or regex (#7350). A `\` before other bytes falls
            // through to the normal arms, so `\(` / `\[` / `\{` still count
            // their brackets exactly as OXC keeps them live.
            b'\\' => {
                if matches!(bytes.get(i + 1), Some(b'\\' | b'\'' | b'"' | b'`' | b'/')) {
                    i += 1;
                } else if track_type_angles && !starts_valid_identifier_escape(bytes, i) {
                    malformed_type_escape_opens += 1;
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
        excessive_speculative_type_angle_opens,
        excessive_arrow_default_speculation,
        oversized_numeric_token,
    }
}
