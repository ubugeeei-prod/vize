//! Reconcile native instance probes with leading authored TS directives.
//!
//! Public-instance reads can be emitted outside their interpolation. Only a
//! successfully parsed interpolation's next physical line can own that read.
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
/// The final leading directive owns only the following authored physical line.
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
        let trimmed = rest.trim_start_matches(|c: char| c.is_ascii_whitespace());
        cursor += rest.len() - trimmed.len();
        let (body, end) = if let Some(after) = trimmed.strip_prefix("//") {
            let length = after.find('\n').unwrap_or(after.len());
            (after.get(..length)?, cursor + 2 + length)
        } else if let Some(after) = trimmed.strip_prefix("/*") {
            let length = after.find("*/")?;
            (after.get(..length)?, cursor + 2 + length + 2)
        } else {
            break;
        };
        let body = body.trim_start();
        let expect = if directive_token(body, "@ts-expect-error") {
            Some(true)
        } else if directive_token(body, "@ts-ignore") {
            Some(false)
        } else {
            None
        };
        if let Some(expect) = expect {
            owner = Some((cursor..end, expect));
        }
        cursor = end;
    }
    let (comment, expect) = owner?;
    // Requiring the next physical line to contain the expression also refuses
    // comments or blank lines between the directive and its proposed target.
    let after_comment = expression.get(comment.end..)?;
    let newline = after_comment.find('\n')?;
    if !after_comment.get(..newline)?.trim().is_empty() {
        return None;
    }
    let target_start = comment.end + newline + 1;
    let target_end = target_start
        + expression
            .get(target_start..)?
            .find('\n')
            .unwrap_or(expression.len() - target_start);
    if cursor < target_start || cursor >= target_end {
        return None;
    }
    Some(Directive {
        comment: base + comment.start..base + comment.end,
        target: base + target_start..base + target_end,
        expect,
        consumed: false,
    })
}

fn directive_token(text: &str, token: &str) -> bool {
    text.strip_prefix(token).is_some_and(|tail| {
        tail.chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '-')
    })
}

#[cfg(test)]
mod tests;
