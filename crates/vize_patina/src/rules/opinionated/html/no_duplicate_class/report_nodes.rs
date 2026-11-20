//! `:class` duplicates, reported on the nodes `vue/no-duplicate-class-names`
//! reports. Every class part keeps its ESTree node range (a quasi spans its
//! `` ` ``, `${` and `}` delimiters) and the range of its ESTree parent, which
//! is `None` for the binding value itself.

use super::{repeated_names, report};
use crate::context::LintContext;
use crate::ir::ByteRange;
use crate::markup::MarkupBinding;
use crate::rules::opinionated::class_values::possible_static_parts;
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;
use vize_l0::{FxHashMap, FxHashSet};

/// A class part's ESTree node range and its parent's range.
type Nodes = FxHashMap<Span, (Span, Option<Span>)>;

/// The first part that rendered a name: its parent, whether it is always
/// rendered, and the `+` or template literal it shares.
struct Collected {
    parent: Option<Span>,
    unconditional: bool,
    group: Option<Span>,
}

pub(super) fn check_binding(
    ctx: &mut LintContext<'_>,
    binding: &MarkupBinding<'_>,
    open_tag: ByteRange,
    static_classes: &str,
) {
    let Some(expression) = binding.retained_expression() else {
        return;
    };
    let container = attribute_value_range(ctx.source, binding.range()).unwrap_or(binding.range());
    let mut parts = Vec::new();
    possible_static_parts(expression.ast, &mut parts);
    let mut nodes = Nodes::default();
    record_nodes(expression.ast, None, &mut nodes);
    let range_of = |parent: Option<Span>| match parent {
        Some(span) => expression.authored_range(span),
        None => Some(container),
    };

    let mut reported = FxHashSet::default();
    let mut repeated_across = Vec::new();
    let mut seen = FxHashMap::<&str, Option<Span>>::default();
    let mut collected = FxHashMap::<&str, Collected>::default();
    for part in &parts {
        let Some(&(node, parent)) = nodes.get(&part.span) else {
            continue;
        };
        let within = repeated_names(part.text);
        if !within.is_empty() {
            if let Some(range) = expression.authored_range(node) {
                report(ctx, range, &within);
            }
            reported.extend(within);
        }
        let names = part.text.split_ascii_whitespace().collect::<Vec<_>>();
        for &name in &names {
            if reported.contains(name) {
                continue;
            }
            if let Some(existing) = collected.get(name) {
                let same_group = part.shared_group.is_some() && part.shared_group == existing.group;
                if (part.unconditional || same_group || existing.unconditional)
                    && !repeated_across.contains(&name)
                {
                    repeated_across.push(name);
                }
            } else {
                collected.insert(
                    name,
                    Collected {
                        parent,
                        unconditional: part.unconditional,
                        group: part.shared_group,
                    },
                );
            }
            if part.unconditional {
                if seen.contains_key(name) {
                    if !repeated_across.contains(&name) {
                        repeated_across.push(name);
                    }
                } else {
                    seen.insert(name, parent);
                }
            }
        }
        let shared = names
            .iter()
            .copied()
            .filter(|name| {
                static_classes
                    .split_ascii_whitespace()
                    .any(|class| class == *name)
            })
            .collect::<Vec<_>>();
        report(ctx, open_tag, &shared);
    }
    for name in repeated_across {
        let parent = seen
            .get(name)
            .copied()
            .or_else(|| collected.get(name).map(|first| first.parent));
        if let Some(range) = parent.and_then(range_of) {
            report(ctx, range, &[name]);
        }
    }
}

/// Map every class part `possible_static_parts` can produce to its ESTree
/// node and parent ranges, walking the same shapes it walks.
fn record_nodes(expression: &Expression<'_>, parent: Option<Span>, nodes: &mut Nodes) {
    match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => {
            nodes.insert(literal.span, (literal.span, parent));
        }
        Expression::TemplateLiteral(template) => {
            for quasi in &template.quasis {
                let close = if quasi.tail { 1 } else { 2 };
                let node = Span::new(quasi.span.start.saturating_sub(1), quasi.span.end + close);
                nodes.insert(quasi.span, (node, Some(template.span)));
            }
            for expression in &template.expressions {
                record_nodes(expression, Some(template.span), nodes);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                if let Some(expression) = element.as_expression() {
                    record_nodes(expression, Some(array.span), nodes);
                }
            }
        }
        Expression::ObjectExpression(object) => {
            for entry in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = entry else {
                    continue;
                };
                let key = match &property.key {
                    PropertyKey::StaticIdentifier(identifier) => Some(identifier.span),
                    PropertyKey::StringLiteral(literal) => Some(literal.span),
                    key => key
                        .as_expression()
                        .map(|key| key.without_parentheses().span()),
                };
                if let Some(key) = key {
                    nodes.insert(key, (key, Some(property.span)));
                }
            }
        }
        Expression::ConditionalExpression(conditional) => {
            record_nodes(&conditional.consequent, Some(conditional.span), nodes);
            record_nodes(&conditional.alternate, Some(conditional.span), nodes);
        }
        Expression::LogicalExpression(logical) => {
            record_nodes(&logical.left, Some(logical.span), nodes);
            record_nodes(&logical.right, Some(logical.span), nodes);
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            record_nodes(&binary.left, Some(binary.span), nodes);
            record_nodes(&binary.right, Some(binary.span), nodes);
        }
        _ => {}
    }
}

/// The authored value of `name="value"`, quotes included, as vue-eslint-parser
/// ranges a `VLiteral` or `VExpressionContainer`.
pub(super) fn attribute_value_range(source: &str, attribute: ByteRange) -> Option<ByteRange> {
    let text = source.get(attribute.start as usize..attribute.end as usize)?;
    let equals = text.find('=')?;
    let after = text.get(equals + 1..)?;
    let value = equals + 1 + (after.len() - after.trim_start().len());
    let rest = text.get(value..)?;
    let length = match rest.as_bytes().first()? {
        quote @ (b'"' | b'\'') => rest.get(1..)?.find(char::from(*quote))? + 2,
        _ => rest
            .find(|character: char| character.is_ascii_whitespace())
            .unwrap_or(rest.len()),
    };
    Some(ByteRange::new(
        attribute.start + value as u32,
        attribute.start + (value + length) as u32,
    ))
}
