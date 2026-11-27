//! Operator-run scan for the expression safety guard.
//!
//! OXC parses each prefix operator by recursing into another unary expression.
//! A deeply repeated run such as thousands of `+` tokens therefore burns parser
//! time without increasing bracket depth, so the nesting guard needs a separate
//! budget for the same safety boundary.
//!
//! The right-associative operators recurse the same way without a bracket or
//! a prefix run: each `?:`, `=>`, assignment and `**` parses its right side one
//! frame deeper. They share one budget for the whole expression.

use super::{MAX_EXPRESSION_NESTING_DEPTH, scan};
use crate::SmallVec;

/// Right-associative operators accepted in one expression. Authored code
/// chains a handful; the parser's stack needs thousands to run out.
const MAX_RIGHT_ASSOCIATIVE_OPERATORS: usize = 256;

/// Whether the `=` at `at` assigns. `==`, `===`, `!=`, `!==`, `<=` and `>=`
/// compare, and `=>` is counted as an arrow by the caller.
fn is_assignment_equals(bytes: &[u8], at: usize) -> bool {
    if bytes.get(at + 1) == Some(&b'=') {
        return false;
    }
    let before = |back: usize| at.checked_sub(back).and_then(|index| bytes.get(index));
    match before(1) {
        Some(b'=' | b'!') => false,
        // `<<=`, `>>=` and `>>>=` assign; a lone `<` or `>` compares.
        Some(angle @ (b'<' | b'>')) => before(2) == Some(angle),
        _ => true,
    }
}

#[inline(always)]
pub(super) fn has_excessive_operator_recursion(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut right_associative_operators = 0usize;
    let mut can_start_operand = true;
    // Template expressions nest a few levels; these stacks stay inline.
    let mut delimiters: SmallVec<[u8; 32]> = SmallVec::new();
    let mut prefix_operator_run = 0usize;
    let mut template_interpolation_depths: SmallVec<[usize; 8]> = SmallVec::new();
    let mut i = 0usize;

    while let Some(&byte) = bytes.get(i) {
        match byte {
            b' ' | b'\t' | b'\r' | b'\n' | 0x0b | 0x0c => {
                i += 1;
            }
            b'"' | b'\'' => {
                i = scan::skip_quoted(bytes, i + 1, byte);
                can_start_operand = false;
                prefix_operator_run = 0;
            }
            b'`' => {
                let (next, has_interpolation) = scan::skip_template_text(bytes, i + 1);
                i = next;
                if has_interpolation {
                    delimiters.push(b'}');
                    template_interpolation_depths.push(delimiters.len());
                    can_start_operand = true;
                } else {
                    can_start_operand = false;
                }
                prefix_operator_run = 0;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i = scan::skip_line_comment(bytes, i + 2);
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = scan::skip_block_comment(bytes, i + 2);
            }
            b'/' if can_start_operand => {
                if let Some(next) = scan::skip_regex(bytes, i + 1) {
                    i = next;
                    can_start_operand = false;
                    prefix_operator_run = 0;
                } else {
                    can_start_operand = true;
                    prefix_operator_run = 0;
                    i += 1;
                }
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                let start = i;
                i = scan::skip_identifier(bytes, i + 1);
                if can_start_operand && is_prefix_keyword(bytes.get(start..i).unwrap_or_default()) {
                    if increment_prefix_operator_run(&mut prefix_operator_run) {
                        return true;
                    }
                    can_start_operand = true;
                } else {
                    can_start_operand =
                        scan::keyword_allows_regex_after(bytes.get(start..i).unwrap_or_default());
                    prefix_operator_run = 0;
                }
            }
            b'0'..=b'9' => {
                i = scan::skip_number(bytes, i + 1);
                can_start_operand = false;
                prefix_operator_run = 0;
            }
            b'+' | b'-' if bytes.get(i + 1) == Some(&byte) => {
                if can_start_operand {
                    if increment_prefix_operator_run(&mut prefix_operator_run)
                        || increment_prefix_operator_run(&mut prefix_operator_run)
                    {
                        return true;
                    }
                    can_start_operand = true;
                } else {
                    can_start_operand = false;
                    prefix_operator_run = 0;
                }
                i += 2;
            }
            b'+' | b'-' | b'!' | b'~' => {
                if can_start_operand {
                    if increment_prefix_operator_run(&mut prefix_operator_run) {
                        return true;
                    }
                } else {
                    prefix_operator_run = 0;
                }
                can_start_operand = true;
                i += 1;
            }
            b'(' | b'[' | b'{' | b'<' | b'@' => {
                if matches!(byte, b'(' | b'[' | b'{') {
                    delimiters.push(match byte {
                        b'(' => b')',
                        b'[' => b']',
                        _ => b'}',
                    });
                }
                can_start_operand = true;
                prefix_operator_run = 0;
                i += 1;
            }
            b'}' if template_interpolation_depths.last() == Some(&delimiters.len()) => {
                delimiters.pop();
                template_interpolation_depths.pop();
                let (next, has_interpolation) = scan::skip_template_text(bytes, i + 1);
                i = next;
                if has_interpolation {
                    delimiters.push(b'}');
                    template_interpolation_depths.push(delimiters.len());
                    can_start_operand = true;
                } else {
                    can_start_operand = false;
                }
                prefix_operator_run = 0;
            }
            b')' | b']' | b'}' | b'.' | b'>' | b'\\' => {
                if matches!(byte, b')' | b']' | b'}') {
                    delimiters.pop();
                }
                can_start_operand = false;
                prefix_operator_run = 0;
                i += 1;
            }
            b',' | b';' | b':' | b'?' | b'=' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^' => {
                let next = bytes.get(i + 1).copied();
                let width = match (byte, next) {
                    // `??` and `?.name` do not recurse; `?.5` is a conditional.
                    (b'?', Some(b'?')) => 2,
                    (b'?', Some(b'.'))
                        if !bytes.get(i + 2).is_some_and(|digit| digit.is_ascii_digit()) =>
                    {
                        1
                    }
                    (b'?', _) => {
                        right_associative_operators += 1;
                        1
                    }
                    (b'=', Some(b'>')) | (b'*', Some(b'*')) => {
                        right_associative_operators += 1;
                        2
                    }
                    (b'=', _) if is_assignment_equals(bytes, i) => {
                        right_associative_operators += 1;
                        1
                    }
                    _ => 1,
                };
                if right_associative_operators > MAX_RIGHT_ASSOCIATIVE_OPERATORS {
                    return true;
                }
                can_start_operand = true;
                prefix_operator_run = 0;
                i += width;
            }
            _ => {
                can_start_operand = false;
                prefix_operator_run = 0;
                i += 1;
            }
        }
    }

    false
}

fn increment_prefix_operator_run(run: &mut usize) -> bool {
    *run += 1;
    *run > MAX_EXPRESSION_NESTING_DEPTH
}

fn is_prefix_keyword(identifier: &[u8]) -> bool {
    matches!(
        identifier,
        b"await" | b"delete" | b"new" | b"typeof" | b"void"
    )
}
