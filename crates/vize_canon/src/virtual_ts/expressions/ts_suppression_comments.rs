use std::borrow::Cow;

use vize_carton::String;
use vize_croquis::drawer::strip_js_comments;

mod callback_line_suppression;
use callback_line_suppression::{CallbackLineRewrite, rewrite_callback_branches};

/// Keep inference-only callback parameters on a line covered by a generated
/// `@ts-ignore` without changing the authored diagnostic owner.
///
/// Ordinary arrows use the allocation-free lexical path. Rare generic,
/// sequence, conditional and function shapes delegate to parser-derived
/// branch-local suppression ranges. `None` leaves borrowed input unchanged.
pub(crate) fn callback_parameters_on_ignored_line(value: &str) -> Option<String> {
    match rewrite_callback_branches(value) {
        CallbackLineRewrite::Rewritten(rewritten) => return Some(rewritten),
        CallbackLineRewrite::Preserve => return None,
        CallbackLineRewrite::NotApplicable => {}
    }

    let arrow = lexical_arrow_token_offset(value)?;
    let prefix = value.get(..arrow)?;
    if !prefix.contains(['\n', '\r']) {
        return None;
    }
    let mut collapsed = String::with_capacity(value.len());
    for ch in prefix.chars() {
        collapsed.push(if matches!(ch, '\n' | '\r') { ' ' } else { ch });
    }
    collapsed.push_str(value.get(arrow..)?);
    Some(collapsed)
}

fn lexical_arrow_token_offset(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut index: usize = 0;
    let mut parenthesis_depth: usize = 0;
    let mut shallowest_arrow: Option<(usize, usize)> = None;
    while bytes.get(index.saturating_add(1)).is_some() {
        let current = bytes.get(index).copied()?;
        match current {
            b'/' if bytes.get(index.saturating_add(1)) == Some(&b'/') => {
                index = index.saturating_add(2);
                while bytes
                    .get(index)
                    .is_some_and(|byte| !matches!(byte, b'\n' | b'\r'))
                {
                    index = index.saturating_add(1);
                }
            }
            b'/' if bytes.get(index.saturating_add(1)) == Some(&b'*') => {
                index = index.saturating_add(2);
                while bytes.get(index.saturating_add(1)).is_some()
                    && !(bytes.get(index) == Some(&b'*')
                        && bytes.get(index.saturating_add(1)) == Some(&b'/'))
                {
                    index = index.saturating_add(1);
                }
                index = index.saturating_add(2).min(bytes.len());
            }
            b'\'' | b'"' | b'`' => {
                index = index.saturating_add(1);
                while bytes.get(index).is_some_and(|byte| *byte != current) {
                    let skip = if bytes.get(index) == Some(&b'\\') {
                        2
                    } else {
                        1
                    };
                    index = index.saturating_add(skip);
                }
                index = index.saturating_add(1).min(bytes.len());
            }
            b'(' => {
                parenthesis_depth = parenthesis_depth.saturating_add(1);
                index = index.saturating_add(1);
            }
            b')' => {
                parenthesis_depth = parenthesis_depth.checked_sub(1)?;
                index = index.saturating_add(1);
            }
            b'=' if bytes.get(index.saturating_add(1)) == Some(&b'>') => {
                if parenthesis_depth == 0 {
                    return Some(index);
                }
                if shallowest_arrow.is_none_or(|(_, depth)| parenthesis_depth < depth) {
                    shallowest_arrow = Some((index, parenthesis_depth));
                }
                index = index.saturating_add(2);
            }
            _ => index = index.saturating_add(1),
        }
    }
    shallowest_arrow.map(|(offset, _)| offset)
}

pub(super) fn expression_source_for_typecheck(expr: &str) -> Cow<'_, str> {
    if contains_typescript_suppression_comment(expr) {
        Cow::Borrowed(expr)
    } else {
        strip_js_comments(expr)
    }
}

fn contains_typescript_suppression_comment(expr: &str) -> bool {
    let mut rest = expr;
    loop {
        rest = rest.trim_start_matches(|ch: char| ch.is_ascii_whitespace());
        let (body, tail) = if let Some(after) = rest.strip_prefix("//") {
            after.split_once('\n').unwrap_or((after, ""))
        } else if let Some(after) = rest.strip_prefix("/*") {
            after.split_once("*/").unwrap_or_else(|| {
                let mut chars = after.chars();
                chars.next_back();
                (chars.as_str(), "")
            })
        } else {
            return false;
        };
        if is_typescript_suppression_directive(body) {
            return true;
        }
        rest = tail;
    }
}

fn is_typescript_suppression_directive(comment: &str) -> bool {
    let trimmed = comment.trim_start();
    trimmed.starts_with("@ts-ignore") || trimmed.starts_with("@ts-expect-error")
}

#[cfg(test)]
mod tests;
