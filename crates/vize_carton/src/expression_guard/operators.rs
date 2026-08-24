//! Operator-run scan for the expression safety guard.
//!
//! OXC parses each prefix operator by recursing into another unary expression.
//! A deeply repeated run such as thousands of `+` tokens therefore burns parser
//! time without increasing bracket depth, so the nesting guard needs a separate
//! budget for the same safety boundary.

use super::{MAX_EXPRESSION_NESTING_DEPTH, scan};

pub(super) fn has_excessive_prefix_operator_run(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut can_start_operand = true;
    let mut delimiters = Vec::new();
    let mut prefix_operator_run = 0usize;
    let mut template_interpolation_depths = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' | b'\r' | b'\n' => {
                i += 1;
            }
            b'"' | b'\'' => {
                i = scan::skip_quoted(bytes, i + 1, bytes[i]);
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
                if can_start_operand && is_prefix_keyword(&bytes[start..i]) {
                    if increment_prefix_operator_run(&mut prefix_operator_run) {
                        return true;
                    }
                    can_start_operand = true;
                } else {
                    can_start_operand = scan::keyword_allows_regex_after(&bytes[start..i]);
                    prefix_operator_run = 0;
                }
            }
            b'0'..=b'9' => {
                i = scan::skip_number(bytes, i + 1);
                can_start_operand = false;
                prefix_operator_run = 0;
            }
            b'+' | b'-' if bytes.get(i + 1) == Some(&bytes[i]) => {
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
                if matches!(bytes[i], b'(' | b'[' | b'{') {
                    delimiters.push(match bytes[i] {
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
                if matches!(bytes[i], b')' | b']' | b'}') {
                    delimiters.pop();
                }
                can_start_operand = false;
                prefix_operator_run = 0;
                i += 1;
            }
            b',' | b';' | b':' | b'?' | b'=' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^' => {
                can_start_operand = true;
                prefix_operator_run = 0;
                i += 1;
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
    matches!(identifier, b"await" | b"delete" | b"typeof" | b"void")
}
