//! The vue-eslint-parser template AST that selectors match, as JSON.
//!
//! Template nodes (`VElement`, `VStartTag`, `VAttribute`, `VDirectiveKey`,
//! `VIdentifier`, `VLiteral`, `VExpressionContainer`, `VText`, `VOnExpression`,
//! `VForExpression`, `VSlotScopeExpression`) are built from the relief tree and
//! the authored text; expressions are OXC's TS-ESTree serialization with
//! parentheses removed. Every node carries authored `start` / `end` offsets.

use crate::context::LintContext;
use serde_json::{Map, Value, json};
use vize_l0::Span;
use vize_l1::markup::entity::EntityContext;
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode,
};

/// The template as one `VElement[name='template']` root.
pub(super) fn template<'a>(ctx: &mut LintContext<'a>, root: &RootNode<'a>) -> Value {
    let end = ctx.source.len();
    let mut children = Vec::new();
    flatten_children(&root.children, &mut children);
    let children = child_nodes(ctx, &children, 0, end);
    node(
        "VElement",
        0,
        end,
        [
            ("name", json!("template")),
            ("rawName", json!("template")),
            ("startTag", Value::Null),
            ("children", Value::Array(children)),
        ],
    )
}

pub(super) fn node<const N: usize>(
    kind: &str,
    start: usize,
    end: usize,
    fields: [(&str, Value); N],
) -> Value {
    let mut map = Map::new();
    map.insert("type".into(), json!(kind));
    for (key, value) in fields {
        map.insert(key.into(), value);
    }
    map.insert("start".into(), json!(start));
    map.insert("end".into(), json!(end));
    Value::Object(map)
}

/// `v-if` / `v-for` wrappers do not exist in the authored tree.
fn flatten_children<'n, 'a>(
    children: &'n [TemplateChildNode<'a>],
    out: &mut Vec<&'n TemplateChildNode<'a>>,
) {
    for child in children {
        match child {
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    flatten_children(&branch.children, out);
                }
            }
            TemplateChildNode::IfBranch(branch) => flatten_children(&branch.children, out),
            TemplateChildNode::For(node) => flatten_children(&node.children, out),
            child => out.push(child),
        }
    }
}

/// Elements and interpolations in order, with the authored text between them
/// as `VText`; comments are tokens, not children.
fn child_nodes<'a>(
    ctx: &mut LintContext<'a>,
    children: &[&TemplateChildNode<'a>],
    start: usize,
    end: usize,
) -> Vec<Value> {
    let mut nodes = Vec::new();
    let mut cursor = start;
    for child in children {
        let (child_start, child_end) = match child {
            TemplateChildNode::Element(element) => {
                let (start, _, _, end) = element_extent(ctx.source, element);
                (start, end)
            }
            TemplateChildNode::Interpolation(interpolation) => span(interpolation.loc.span),
            TemplateChildNode::Comment(comment) => span(comment.loc.span),
            _ => continue,
        };
        push_text(ctx, &mut nodes, cursor, child_start);
        cursor = child_end.max(cursor);
        match child {
            TemplateChildNode::Element(element) => nodes.push(element_node(ctx, element)),
            TemplateChildNode::Interpolation(interpolation) => {
                let expression = expression::expression_value(
                    ctx,
                    &interpolation.content,
                    EntityContext::Text,
                    None,
                );
                nodes.push(node(
                    "VExpressionContainer",
                    child_start,
                    child_end,
                    [("expression", expression)],
                ));
            }
            _ => {}
        }
    }
    push_text(ctx, &mut nodes, cursor, end);
    nodes
}

fn push_text(ctx: &LintContext<'_>, nodes: &mut Vec<Value>, start: usize, end: usize) {
    if let Some(text) = ctx.source.get(start..end).filter(|text| !text.is_empty()) {
        nodes.push(node("VText", start, end, [("value", json!(text))]));
    }
}

pub(super) fn span(span: Span) -> (usize, usize) {
    (span.start as usize, span.end as usize)
}

/// Relief stores only the start tag on `ElementNode::loc`. The selector tree
/// uses the authored element, through its closing tag when one is written.
fn element_extent(source: &str, element: &ElementNode<'_>) -> (usize, usize, usize, usize) {
    let (start, open_end) = span(element.loc.span);
    if element.is_self_closing {
        return (start, open_end, open_end, open_end);
    }
    let Some(closing) =
        super::super::super::authored_closing::find(source, element.loc.span.end, element.tag)
    else {
        return (start, open_end, open_end, open_end);
    };
    (
        start,
        open_end,
        closing.start as usize,
        closing.end as usize,
    )
}

fn element_node<'a>(ctx: &mut LintContext<'a>, element: &ElementNode<'a>) -> Value {
    let (start, open_end, content_end, end) = element_extent(ctx.source, element);
    let attributes = element
        .props
        .iter()
        .map(|prop| match prop {
            PropNode::Attribute(attribute) => {
                let (attribute_start, attribute_end) = span(attribute.loc.span);
                let (name_start, name_end) = span(attribute.name_loc.span);
                let value = attribute.value.as_ref().map_or(Value::Null, |value| {
                    let (value_start, value_end) = quoted(ctx.source, span(value.loc.span));
                    node(
                        "VLiteral",
                        value_start,
                        value_end,
                        [("value", json!(value.content))],
                    )
                });
                node(
                    "VAttribute",
                    attribute_start,
                    attribute_end,
                    [
                        ("directive", json!(false)),
                        ("key", identifier(attribute.name, name_start, name_end)),
                        ("value", value),
                    ],
                )
            }
            PropNode::Directive(directive) => directive_node(ctx, directive),
        })
        .collect::<Vec<_>>();
    let mut children = Vec::new();
    flatten_children(&element.children, &mut children);
    let children = child_nodes(ctx, &children, open_end, content_end);
    node(
        "VElement",
        start,
        end,
        [
            ("name", json!(element.tag.to_ascii_lowercase())),
            ("rawName", json!(element.tag)),
            (
                "startTag",
                node(
                    "VStartTag",
                    start,
                    open_end.max(start),
                    [("attributes", Value::Array(attributes))],
                ),
            ),
            ("children", Value::Array(children)),
        ],
    )
}

fn identifier(raw: &str, start: usize, end: usize) -> Value {
    node(
        "VIdentifier",
        start,
        end,
        [
            ("name", json!(raw.to_ascii_lowercase())),
            ("rawName", json!(raw)),
        ],
    )
}

/// A value range widened over its quotes.
fn quoted(source: &str, (start, end): (usize, usize)) -> (usize, usize) {
    let before = start
        .checked_sub(1)
        .and_then(|at| source.as_bytes().get(at));
    match before {
        Some(b'"' | b'\'') => (start - 1, (end + 1).min(source.len())),
        _ => (start, end),
    }
}

fn directive_node<'a>(ctx: &mut LintContext<'a>, directive: &DirectiveNode<'a>) -> Value {
    let (start, end) = span(directive.loc.span);
    let raw = directive.raw_name.unwrap_or(directive.name);
    let shorthand = ["v-", ":", "@", "#", "."]
        .into_iter()
        .find(|prefix| raw.starts_with(prefix));
    let name_raw = match shorthand {
        Some("v-") | None => directive.name,
        Some(prefix) => prefix,
    };
    let argument = directive
        .arg
        .as_ref()
        .map_or(Value::Null, |argument| match argument {
            ExpressionNode::Simple(simple) if simple.is_static => {
                let (arg_start, arg_end) = span(simple.loc.span);
                identifier(simple.content, arg_start, arg_end)
            }
            argument => {
                let (arg_start, arg_end) = expression::expression_span(argument);
                let expression =
                    expression::expression_value(ctx, argument, EntityContext::Attribute, None);
                node(
                    "VExpressionContainer",
                    arg_start.saturating_sub(1),
                    arg_end + 1,
                    [("expression", expression)],
                )
            }
        });
    let value = directive.exp.as_ref().map_or(Value::Null, |expression| {
        let (value_start, value_end) = quoted(ctx.source, expression::expression_span(expression));
        let expression = expression::expression_value(
            ctx,
            expression,
            EntityContext::Attribute,
            Some(directive.name),
        );
        node(
            "VExpressionContainer",
            value_start,
            value_end,
            [("expression", expression)],
        )
    });
    let key_end = directive
        .exp
        .as_ref()
        .map_or(end, |expression| {
            expression::expression_span(expression).0.saturating_sub(2)
        })
        .max(start);
    node(
        "VAttribute",
        start,
        end,
        [
            ("directive", json!(true)),
            (
                "key",
                node(
                    "VDirectiveKey",
                    start,
                    key_end,
                    [
                        (
                            "name",
                            node(
                                "VIdentifier",
                                start,
                                start + name_raw.len(),
                                [
                                    ("name", json!(directive.name)),
                                    ("rawName", json!(name_raw)),
                                ],
                            ),
                        ),
                        ("argument", argument),
                    ],
                ),
            ),
            ("value", value),
        ],
    )
}

mod expression;
