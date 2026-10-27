//! Reconcile native instance probes with leading authored TS directives.
//!
//! Public-instance reads can be emitted outside their interpolation. Only a
//! successfully parsed interpolation's next code line can own that read.
use std::ops::Range;

use vize_atelier_core::{ParserOptions, parser::parse_with_options};
use vize_carton::Allocator;
use vize_relief::{ExpressionNode, TemplateChildNode};

use super::super::{AssembledDiagnostic, AssembledOrigin, AuthoredSource};

struct Directive {
    comment: Range<usize>,
    target: Range<usize>,
    expect: bool,
    consumed: bool,
}

pub(super) fn filter<P>(
    authored: &AuthoredSource<'_>,
    diagnostics: &mut Vec<AssembledDiagnostic<P>>,
) {
    if !authored.is_vue
        || !(authored.text.contains("@ts-expect-error") || authored.text.contains("@ts-ignore"))
        || !diagnostics
            .iter()
            .any(|d| d.code == Some(2339) && matches!(&d.origin, AssembledOrigin::Checker(_)))
    {
        return;
    }
    let Some(template) = authored.template() else {
        return;
    };
    let Some(text) = authored.text.get(template.clone()) else {
        return;
    };
    let allocator = Allocator::new();
    let (root, errors) = parse_with_options(&allocator, text, ParserOptions::default());
    if !errors.is_empty() {
        return;
    }
    let mut directives = Vec::new();
    let mut nodes: Vec<_> = root.children.iter().collect();
    while let Some(node) = nodes.pop() {
        match node {
            TemplateChildNode::Element(element) => nodes.extend(element.children.iter()),
            TemplateChildNode::Interpolation(interpolation) => {
                let ExpressionNode::Simple(expression) = &interpolation.content else {
                    continue;
                };
                let start = template.start + expression.loc.span.start as usize;
                let end = template.start + expression.loc.span.end as usize;
                // Transformed or approximate locations never authorize suppression.
                if authored.text.get(start..end) == Some(expression.content)
                    && let Some(directive) = leading_directive(expression.content, start)
                {
                    directives.push(directive);
                }
            }
            _ => {}
        }
    }
    directives.sort_unstable_by_key(|d| d.target.start);
    diagnostics.retain(|diagnostic| {
        if diagnostic.code != Some(2339)
            || !matches!(&diagnostic.origin, AssembledOrigin::Checker(_))
        {
            return true;
        }
        let index = directives.partition_point(|d| d.target.start <= diagnostic.start);
        let Some(owner) = index.checked_sub(1).and_then(|i| directives.get_mut(i)) else {
            return true;
        };
        if owner.target.contains(&diagnostic.start) {
            owner.consumed = true;
            false
        } else {
            true
        }
    });
    diagnostics.retain(|diagnostic| {
        if diagnostic.code != Some(2578)
            || !matches!(&diagnostic.origin, AssembledOrigin::Checker(_))
            || !diagnostic
                .message
                .contains("Unused '@ts-expect-error' directive")
        {
            return true;
        }
        let index = directives.partition_point(|d| d.comment.start <= diagnostic.start);
        !index
            .checked_sub(1)
            .and_then(|i| directives.get(i))
            .is_some_and(|d| d.expect && d.consumed && d.comment.contains(&diagnostic.start))
    });
}

/// Recognize leading comments, never strings, attributes or trailing comments.
/// The final leading directive owns only the following authored code line.
fn leading_directive(expression: &str, base: usize) -> Option<Directive> {
    if !(expression.contains("@ts-ignore") || expression.contains("@ts-expect-error"))
        || !vize_l0::expression_guard::expression_is_safe_to_parse(expression)
    {
        return None;
    }
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, expression, oxc_span::SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let mut cursor = 0;
    let mut owner = None;
    loop {
        let rest = expression.get(cursor..)?;
        let trimmed = rest.trim_start_matches(is_js_whitespace);
        cursor += rest.len() - trimmed.len();
        let (body, end, block) = if let Some(after) = trimmed.strip_prefix("//") {
            let length = after.find(is_line_terminator).unwrap_or(after.len());
            (after.get(..length)?, cursor + 2 + length, false)
        } else if let Some(after) = trimmed.strip_prefix("/*") {
            let length = after.find("*/")?;
            (after.get(..length)?, cursor + 2 + length + 2, true)
        } else {
            break;
        };
        let recognized = if block {
            // The native scanner recognizes directives only on the comment's
            // final physical line, after optional slash/star prefix characters.
            let last_line = body.rsplit(is_line_terminator).next()?;
            directive_kind(
                last_line
                    .trim_start_matches(is_js_whitespace)
                    .trim_start_matches(['/', '*'])
                    .trim_start_matches(is_js_whitespace),
            )
        } else {
            directive_kind(
                body.strip_prefix('/')
                    .unwrap_or(body)
                    .trim_start_matches(is_js_whitespace),
            )
        };
        if let Some(expect) = recognized {
            owner = Some((cursor..end, expect));
        }
        cursor = end;
    }
    let (comment, expect) = owner?;
    // The leading-comment walk admits only whitespace and complete comments
    // between this directive and the first code token. TypeScript permits
    // blank lines and explanatory comments here; later code lines are separate.
    let (line_start, terminator) = expression
        .get(..cursor)?
        .char_indices()
        .rev()
        .find(|(_, c)| is_line_terminator(*c))?;
    let target_start = line_start + terminator.len_utf8();
    if comment.end >= target_start {
        return None;
    }
    let after_comment = expression.get(comment.end..target_start)?;
    let (newline, terminator) = after_comment
        .char_indices()
        .find(|(_, c)| is_line_terminator(*c))?;
    // Native lookup walks backward from the line BEFORE the diagnostic:
    // blank and // lines are transparent; block-comment lines are barriers.
    // Comments on the target line itself cannot block its own directive.
    for line in after_comment
        .get(newline + terminator.len_utf8()..)?
        .split(is_line_terminator)
    {
        let line = line.trim_matches(is_js_whitespace);
        if !line.is_empty() && !line.starts_with("//") {
            return None;
        }
    }
    let target_end = cursor
        + expression
            .get(cursor..)?
            .find(is_line_terminator)
            .unwrap_or(expression.len() - cursor);
    if cursor >= target_end {
        return None;
    }
    Some(Directive {
        comment: base + comment.start..base + comment.end,
        target: base + target_start..base + target_end,
        expect,
        consumed: false,
    })
}

fn is_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

fn is_js_whitespace(c: char) -> bool {
    is_line_terminator(c)
        || matches!(
            c,
            '\t' | '\u{000b}' | '\u{000c}' | ' ' | '\u{00a0}' | '\u{1680}' | '\u{2000}'
                ..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
        )
}

fn directive_kind(text: &str) -> Option<bool> {
    if text.starts_with("@ts-expect-error") {
        Some(true)
    } else if text.starts_with("@ts-ignore") {
        Some(false)
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
