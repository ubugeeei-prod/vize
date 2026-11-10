//! Semantic equality checks that keep TSRX layout structure-preserving.

use oxc_span::ContentEq;
use vize_l0::{Span, String, ToCompactString};
use vize_l1::markup::tsrx::{self, Attribute, AttributeValue, Block, Child, Node, TagName};

use crate::FormatError;

use super::normalize_text;

pub(in crate::tsrx) fn require_same_structure(
    before: &tsrx::TsrxModule<'_>,
    before_source: &str,
    after: &tsrx::TsrxModule<'_>,
    after_source: &str,
) -> Result<(), FormatError> {
    if !before.program().content_eq(after.program())
        || before.regions().len() != after.regions().len()
        || !before
            .regions()
            .iter()
            .zip(after.regions())
            .all(|(left, right)| {
                left.site == right.site
                    && node_eq(&left.node, before_source, &right.node, after_source)
            })
    {
        return Err(FormatError::ScriptFormatError(
            "TSRX layout changed the parsed L1 structure".into(),
        ));
    }
    Ok(())
}

fn node_eq(left: &Node<'_>, left_source: &str, right: &Node<'_>, right_source: &str) -> bool {
    match (left, right) {
        (Node::Element(left), Node::Element(right)) => {
            tag_eq(&left.name, left_source, &right.name, right_source)
                && left.attributes.len() == right.attributes.len()
                && left
                    .attributes
                    .iter()
                    .zip(&right.attributes)
                    .all(|(left, right)| attr_eq(left, left_source, right, right_source))
                && children_eq(&left.children, left_source, &right.children, right_source)
                && option_text_eq(left.raw_text, left_source, right.raw_text, right_source)
                && left.closing.is_some() == right.closing.is_some()
        }
        (Node::Fragment(left), Node::Fragment(right)) => {
            children_eq(&left.children, left_source, &right.children, right_source)
        }
        (Node::CodeBlock(left), Node::CodeBlock(right)) => {
            block_eq(left, left_source, right, right_source)
        }
        (Node::If(left), Node::If(right)) => {
            left.branches.len() == right.branches.len()
                && left
                    .branches
                    .iter()
                    .zip(&right.branches)
                    .all(|(left, right)| {
                        option_content_eq(left.test.as_ref(), right.test.as_ref())
                            && block_eq(&left.body, left_source, &right.body, right_source)
                    })
        }
        (Node::For(left), Node::For(right)) => {
            left.r#await == right.r#await
                && left.left.content_eq(&right.left)
                && left.right.content_eq(&right.right)
                && option_content_eq(left.index.as_ref(), right.index.as_ref())
                && option_content_eq(left.key.as_ref(), right.key.as_ref())
                && block_eq(&left.body, left_source, &right.body, right_source)
                && option_block_eq(
                    left.empty.as_ref(),
                    left_source,
                    right.empty.as_ref(),
                    right_source,
                )
        }
        (Node::Switch(left), Node::Switch(right)) => {
            left.discriminant.content_eq(&right.discriminant)
                && left.cases.len() == right.cases.len()
                && left.cases.iter().zip(&right.cases).all(|(left, right)| {
                    option_content_eq(left.test.as_ref(), right.test.as_ref())
                        && block_eq(&left.body, left_source, &right.body, right_source)
                })
        }
        (Node::Try(left), Node::Try(right)) => {
            block_eq(&left.body, left_source, &right.body, right_source)
                && option_block_eq(
                    left.pending.as_ref(),
                    left_source,
                    right.pending.as_ref(),
                    right_source,
                )
                && match (&left.catch, &right.catch) {
                    (Some(left), Some(right)) => {
                        option_content_eq(left.params.as_deref(), right.params.as_deref())
                            && block_eq(&left.body, left_source, &right.body, right_source)
                    }
                    (None, None) => true,
                    _ => false,
                }
        }
        _ => false,
    }
}

fn block_eq(left: &Block<'_>, left_source: &str, right: &Block<'_>, right_source: &str) -> bool {
    left.statements.content_eq(&right.statements)
        && match (&left.render, &right.render) {
            (Some(left), Some(right)) => node_eq(left, left_source, right, right_source),
            (None, None) => true,
            _ => false,
        }
}

fn option_block_eq(
    left: Option<&Block<'_>>,
    left_source: &str,
    right: Option<&Block<'_>>,
    right_source: &str,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => block_eq(left, left_source, right, right_source),
        (None, None) => true,
        _ => false,
    }
}

fn children_eq(
    left: &[Child<'_>],
    left_source: &str,
    right: &[Child<'_>],
    right_source: &str,
) -> bool {
    let left = semantic_children(left, left_source);
    let right = semantic_children(right, right_source);
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| match (left, right) {
                (SemanticChild::Text(left), SemanticChild::Text(right)) => left == right,
                (SemanticChild::Comment(left), SemanticChild::Comment(right)) => left == right,
                (SemanticChild::Expression(left), SemanticChild::Expression(right)) => {
                    option_content_eq(left.expression.as_ref(), right.expression.as_ref())
                }
                (SemanticChild::Node(left), SemanticChild::Node(right)) => {
                    node_eq(left, left_source, right, right_source)
                }
                _ => false,
            })
}

enum SemanticChild<'a, 'b> {
    Text(String),
    Comment(String),
    Expression(&'b tsrx::ExpressionChild<'a>),
    Node(&'b Node<'a>),
}

fn semantic_children<'a, 'b>(
    children: &'b [Child<'a>],
    source: &str,
) -> std::vec::Vec<SemanticChild<'a, 'b>> {
    children
        .iter()
        .filter_map(|child| match child {
            Child::Text(span) => {
                let text = source.get(span.start as usize..span.end as usize)?.trim();
                (!text.is_empty()).then(|| SemanticChild::Text(normalize_text(text)))
            }
            Child::Comment(comment) => source
                .get(comment.span.start as usize..comment.span.end as usize)
                .map(str::trim)
                .map(ToCompactString::to_compact_string)
                .map(SemanticChild::Comment),
            Child::Expression(expression) => Some(SemanticChild::Expression(expression)),
            Child::Node(node) => Some(SemanticChild::Node(node)),
        })
        .collect()
}

fn attr_eq(
    left: &Attribute<'_>,
    left_source: &str,
    right: &Attribute<'_>,
    right_source: &str,
) -> bool {
    match (left, right) {
        (
            Attribute::Named {
                name: left_name,
                value: left_value,
                ..
            },
            Attribute::Named {
                name: right_name,
                value: right_value,
                ..
            },
        ) => {
            text(left_source, *left_name) == text(right_source, *right_name)
                && match (left_value, right_value) {
                    (Some(AttributeValue::String(left)), Some(AttributeValue::String(right))) => {
                        text(left_source, *left) == text(right_source, *right)
                    }
                    (
                        Some(AttributeValue::Expression {
                            expression: left, ..
                        }),
                        Some(AttributeValue::Expression {
                            expression: right, ..
                        }),
                    ) => left.content_eq(right),
                    (None, None) => true,
                    _ => false,
                }
        }
        (
            Attribute::Shorthand {
                expression: left, ..
            },
            Attribute::Shorthand {
                expression: right, ..
            },
        )
        | (
            Attribute::Spread {
                expression: left, ..
            },
            Attribute::Spread {
                expression: right, ..
            },
        ) => left.content_eq(right),
        _ => false,
    }
}

fn tag_eq(left: &TagName<'_>, left_source: &str, right: &TagName<'_>, right_source: &str) -> bool {
    match (left, right) {
        (TagName::Static(left), TagName::Static(right)) => {
            text(left_source, *left) == text(right_source, *right)
        }
        (
            TagName::Dynamic {
                expression: left, ..
            },
            TagName::Dynamic {
                expression: right, ..
            },
        ) => left.content_eq(right),
        _ => false,
    }
}

fn option_text_eq(
    left: Option<Span>,
    left_source: &str,
    right: Option<Span>,
    right_source: &str,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => text(left_source, left) == text(right_source, right),
        (None, None) => true,
        _ => false,
    }
}

fn option_content_eq<T: ContentEq>(left: Option<&T>, right: Option<&T>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.content_eq(right),
        (None, None) => true,
        _ => false,
    }
}

fn text(source: &str, span: Span) -> &str {
    source
        .get(span.start as usize..span.end as usize)
        .unwrap_or_default()
}
