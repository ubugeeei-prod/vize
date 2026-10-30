use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

mod candidate;
use candidate::{CandidateGate, candidate_gate};

#[cfg(test)]
pub(super) fn has_callback_parser_candidate(value: &str) -> bool {
    matches!(candidate_gate(value), CandidateGate::Parse)
}

pub(super) const BRANCH_IGNORE: &str = "// @ts-ignore Inference-only callback branch.\n";

pub(super) struct RewrittenCallbackBranches {
    pub(super) text: String,
    /// Generated-only byte ranges in `text` inserted by this rewrite.
    pub(super) synthetic_ranges: Vec<Range<usize>>,
}

pub(super) enum CallbackLineRewrite {
    NotApplicable,
    Preserve,
    Rewritten(RewrittenCallbackBranches),
}

pub(super) fn rewrite_callback_branches(value: &str) -> CallbackLineRewrite {
    match candidate_gate(value) {
        CandidateGate::Lexical => return CallbackLineRewrite::NotApplicable,
        CandidateGate::Preserve => return CallbackLineRewrite::Preserve,
        CandidateGate::Parse => {}
    }
    let Some(ranges) = direct_callback_prefix_ranges(value) else {
        return CallbackLineRewrite::Preserve;
    };
    if !ranges.iter().any(|range| {
        value
            .get(..range.end)
            .is_some_and(|prefix| prefix.contains(['\n', '\r']))
    }) {
        return CallbackLineRewrite::Preserve;
    }
    rewrite_ranges_with_synthetic_ranges(value, &ranges).map_or(
        CallbackLineRewrite::Preserve,
        CallbackLineRewrite::Rewritten,
    )
}

fn direct_callback_prefix_ranges(value: &str) -> Option<Vec<Range<usize>>> {
    let allocator = Allocator::default();
    let expression = Parser::new(&allocator, value, SourceType::ts())
        .parse_expression()
        .or_else(|_| Parser::new(&allocator, value, SourceType::tsx()).parse_expression())
        .ok()?;
    let mut ranges = Vec::with_capacity(2);
    collect_ranges(&expression, value, &mut ranges)?;
    if ranges.is_empty() {
        return None;
    }
    ranges.sort_unstable_by_key(|range| range.start);
    (!ranges.windows(2).any(|pair| {
        pair.first()
            .zip(pair.get(1))
            .is_some_and(|(left, right)| left.end > right.start)
    }))
    .then_some(ranges)
}

fn collect_ranges(
    expression: &Expression<'_>,
    value: &str,
    ranges: &mut Vec<Range<usize>>,
) -> Option<()> {
    match expression {
        Expression::ArrowFunctionExpression(arrow) => {
            let range_start = usize::try_from(arrow.span.start).ok()?;
            let search_start = usize::try_from(arrow.params.span.end).ok()?;
            let search_end = usize::try_from(arrow.body.span.start).ok()?;
            let range_end = last_structural_arrow(value, search_start, search_end)?;
            (range_start <= range_end).then(|| ranges.push(range_start..range_end))
        }
        Expression::FunctionExpression(function) => {
            let range_start = usize::try_from(function.span.start).ok()?;
            let range_end = usize::try_from(function.params.span.end).ok()?;
            (range_start <= range_end && value.get(range_start..range_end).is_some())
                .then(|| ranges.push(range_start..range_end))
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_ranges(&parenthesized.expression, value, ranges)
        }
        Expression::TSAsExpression(assertion) => {
            collect_ranges(&assertion.expression, value, ranges)
        }
        Expression::TSSatisfiesExpression(assertion) => {
            collect_ranges(&assertion.expression, value, ranges)
        }
        Expression::TSTypeAssertion(assertion) => {
            collect_ranges(&assertion.expression, value, ranges)
        }
        Expression::TSNonNullExpression(non_null) => {
            collect_ranges(&non_null.expression, value, ranges)
        }
        Expression::ConditionalExpression(conditional) => {
            collect_ranges(&conditional.consequent, value, ranges)?;
            collect_ranges(&conditional.alternate, value, ranges)
        }
        Expression::SequenceExpression(sequence) => {
            collect_ranges(sequence.expressions.last()?, value, ranges)
        }
        _ => None,
    }
}

#[cfg(test)]
pub(super) fn rewrite_ranges(value: &str, ranges: &[Range<usize>]) -> Option<String> {
    rewrite_ranges_with_synthetic_ranges(value, ranges).map(|rewrite| rewrite.text)
}

fn rewrite_ranges_with_synthetic_ranges(
    value: &str,
    ranges: &[Range<usize>],
) -> Option<RewrittenCallbackBranches> {
    let inserted = ranges.len().checked_mul(BRANCH_IGNORE.len())?;
    let mut output = String::with_capacity(value.len().checked_add(inserted)?);
    let mut synthetic_ranges = Vec::with_capacity(ranges.len());
    let mut copied = 0usize;
    for range in ranges {
        if copied > range.start || range.start > range.end {
            return None;
        }
        let prefix = value.get(range.clone())?;
        if !safe_to_collapse(prefix) {
            return None;
        }
        output.push_str(value.get(copied..range.start)?);
        let synthetic_start = output.len();
        output.push_str(BRANCH_IGNORE);
        synthetic_ranges.push(synthetic_start..output.len());
        for ch in prefix.chars() {
            output.push(if matches!(ch, '\n' | '\r') { ' ' } else { ch });
        }
        copied = range.end;
    }
    output.push_str(value.get(copied..)?);
    Some(RewrittenCallbackBranches {
        text: output,
        synthetic_ranges,
    })
}

/// A newline in a literal changes its value; one in a line comment terminates
/// that comment. Both reject the rewrite instead of changing semantics.
fn safe_to_collapse(prefix: &str) -> bool {
    safe_prefix(prefix).unwrap_or(false)
}

fn safe_prefix(prefix: &str) -> Option<bool> {
    let bytes = prefix.as_bytes();
    let mut index = 0usize;
    let (mut line_comment, mut block_comment) = (false, false);
    while let Some(&current) = bytes.get(index) {
        let next = index.checked_add(1);
        if line_comment {
            if matches!(current, b'\n' | b'\r') {
                return Some(false);
            }
        } else if block_comment {
            if current == b'*' && next.and_then(|next| bytes.get(next)) == Some(&b'/') {
                block_comment = false;
                let Some(after) = next.and_then(|next| next.checked_add(1)) else {
                    return Some(false);
                };
                index = after;
                continue;
            }
        } else if current == b'/' && next.and_then(|next| bytes.get(next)) == Some(&b'/') {
            line_comment = true;
            index = next.and_then(|next| next.checked_add(1))?;
            continue;
        } else if current == b'/' && next.and_then(|next| bytes.get(next)) == Some(&b'*') {
            block_comment = true;
            index = next.and_then(|next| next.checked_add(1))?;
            continue;
        } else if matches!(current, b'\'' | b'"') {
            index = skip_quoted(bytes, index, current)?;
            continue;
        } else if current == b'`' {
            index = skip_template(bytes, index)?;
            continue;
        }
        index = index.checked_add(1)?;
    }
    Some(!line_comment && !block_comment)
}

fn skip_quoted(bytes: &[u8], mut index: usize, delimiter: u8) -> Option<usize> {
    index = index.checked_add(1)?;
    while let Some(&current) = bytes.get(index) {
        if matches!(current, b'\n' | b'\r') {
            return None;
        }
        if current == b'\\' {
            index = index.checked_add(2)?;
        } else if current == delimiter {
            return index.checked_add(1);
        } else {
            index = index.checked_add(1)?;
        }
    }
    None
}

fn skip_template(bytes: &[u8], mut index: usize) -> Option<usize> {
    index = index.checked_add(1)?;
    while let Some(&current) = bytes.get(index) {
        if matches!(current, b'\n' | b'\r') {
            return None;
        }
        if current == b'\\' {
            index = index.checked_add(2)?;
        } else if current == b'`' {
            return index.checked_add(1);
        } else if current == b'$' && bytes.get(index.checked_add(1)?) == Some(&b'{') {
            index = skip_template_expression(bytes, index.checked_add(2)?, 1)?;
        } else {
            index = index.checked_add(1)?;
        }
    }
    None
}

fn skip_template_expression(bytes: &[u8], mut index: usize, mut depth: usize) -> Option<usize> {
    let (mut line_comment, mut block_comment) = (false, false);
    while let Some(&current) = bytes.get(index) {
        let next = index.checked_add(1)?;
        if line_comment {
            if matches!(current, b'\n' | b'\r') {
                return None;
            }
        } else if block_comment {
            if current == b'*' && bytes.get(next) == Some(&b'/') {
                block_comment = false;
                index = next.checked_add(1)?;
                continue;
            }
        } else if current == b'/' && bytes.get(next) == Some(&b'/') {
            line_comment = true;
            index = next.checked_add(1)?;
            continue;
        } else if current == b'/' && bytes.get(next) == Some(&b'*') {
            block_comment = true;
            index = next.checked_add(1)?;
            continue;
        } else if matches!(current, b'\'' | b'"') {
            index = skip_quoted(bytes, index, current)?;
            continue;
        } else if current == b'`' {
            index = skip_template(bytes, index)?;
            continue;
        } else if current == b'{' {
            depth = depth.checked_add(1)?;
        } else if current == b'}' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return index.checked_add(1);
            }
        }
        index = index.checked_add(1)?;
    }
    None
}

fn last_structural_arrow(value: &str, start: usize, end: usize) -> Option<usize> {
    let bytes = value.as_bytes();
    bytes.get(start..end)?;
    let (mut index, mut last) = (start, None);
    while index < end {
        let current = *bytes.get(index)?;
        let next = index.checked_add(1)?;
        match current {
            b'/' if next < end && bytes.get(next) == Some(&b'/') => {
                index = next.checked_add(1)?;
                while index < end
                    && bytes
                        .get(index)
                        .is_some_and(|byte| !matches!(byte, b'\n' | b'\r'))
                {
                    index = index.checked_add(1)?;
                }
            }
            b'/' if next < end && bytes.get(next) == Some(&b'*') => {
                index = next.checked_add(1)?;
                loop {
                    let comment_next = index.checked_add(1)?;
                    if comment_next >= end {
                        return None;
                    }
                    if bytes.get(index) == Some(&b'*') && bytes.get(comment_next) == Some(&b'/') {
                        index = comment_next.checked_add(1)?;
                        break;
                    }
                    index = index.checked_add(1)?;
                }
            }
            b'\'' | b'"' | b'`' => {
                index = next;
                loop {
                    if index >= end {
                        return None;
                    }
                    let quoted = *bytes.get(index)?;
                    index = index.checked_add(1)?;
                    if quoted == b'\\' {
                        index = index.checked_add(1)?;
                    } else if quoted == current {
                        break;
                    }
                }
            }
            b'=' if next < end && bytes.get(next) == Some(&b'>') => {
                last = Some(index);
                index = next.checked_add(1)?;
            }
            _ => index = index.checked_add(1)?,
        }
    }
    last
}
