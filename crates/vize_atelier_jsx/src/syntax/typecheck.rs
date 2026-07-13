//! Exact owned embedded-expression projection for plain-TypeScript consumers.

use super::{
    JsxSyntaxAttribute, JsxSyntaxAttributeValue, JsxSyntaxExpression, JsxSyntaxNode,
    JsxSyntaxRootMetadata, JsxSyntaxSpan,
};

/// Authored expression text and its exact byte range in the JSX module.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsxTypecheckExpression {
    pub code: Box<str>,
    pub span: JsxSyntaxSpan,
}

/// One type-checkable unit emitted from a JSX render root.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JsxTypecheckEmit {
    Expression(JsxTypecheckExpression),
    ModelTarget(JsxTypecheckExpression),
    ForScope {
        source: JsxTypecheckExpression,
        value: Option<JsxTypecheckExpression>,
        index: Option<JsxTypecheckExpression>,
        body: Vec<JsxTypecheckEmit>,
    },
}

/// One outermost JSX root and the expressions that must remain type-checkable.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JsxTypecheckRoot {
    pub span: JsxSyntaxSpan,
    pub emits: Vec<JsxTypecheckEmit>,
}

pub(super) fn project_roots(
    roots: &[JsxSyntaxNode],
    metadata: &[JsxSyntaxRootMetadata],
) -> Vec<JsxTypecheckRoot> {
    roots
        .iter()
        .zip(metadata)
        .map(|(root, metadata)| {
            let mut emits = Vec::new();
            collect_node(root, &mut emits);
            for style in &metadata.scoped_style_exprs {
                push_expression(&style.content, style.start, style.end, &mut emits);
            }
            JsxTypecheckRoot {
                span: metadata.span,
                emits,
            }
        })
        .collect()
}

fn collect_node(node: &JsxSyntaxNode, out: &mut Vec<JsxTypecheckEmit>) {
    match node {
        JsxSyntaxNode::Element(element) => {
            for attribute in &element.attributes {
                collect_attribute(attribute, out);
            }
            for child in &element.children {
                collect_node(child, out);
            }
        }
        JsxSyntaxNode::Fragment { children, .. } => {
            for child in children {
                collect_node(child, out);
            }
        }
        JsxSyntaxNode::Expression { expression, .. } => {
            out.push(JsxTypecheckEmit::Expression(project(expression)));
        }
        JsxSyntaxNode::If { branches, .. } => {
            for branch in branches {
                if let Some(condition) = &branch.condition {
                    out.push(JsxTypecheckEmit::Expression(project(condition)));
                }
                for child in &branch.body {
                    collect_node(child, out);
                }
            }
        }
        JsxSyntaxNode::For {
            source,
            value,
            index,
            body,
            ..
        } => {
            let mut nested = Vec::new();
            for child in body {
                collect_node(child, &mut nested);
            }
            out.push(JsxTypecheckEmit::ForScope {
                source: project(source),
                value: value.as_ref().map(|binding| JsxTypecheckExpression {
                    code: binding.pattern.clone(),
                    span: binding.span,
                }),
                index: index.as_ref().map(|binding| JsxTypecheckExpression {
                    code: binding.pattern.clone(),
                    span: binding.span,
                }),
                body: nested,
            });
        }
        JsxSyntaxNode::Text { .. } | JsxSyntaxNode::Comment { .. } => {}
    }
}

fn collect_attribute(attribute: &JsxSyntaxAttribute, out: &mut Vec<JsxTypecheckEmit>) {
    match attribute {
        JsxSyntaxAttribute::Spread { expression, .. } => {
            out.push(JsxTypecheckEmit::Expression(project(expression)));
        }
        JsxSyntaxAttribute::Attribute { name, value, .. } => {
            let JsxSyntaxAttributeValue::Expression(expression) = value else {
                return;
            };
            if name.starts_with("v-model") {
                out.push(JsxTypecheckEmit::ModelTarget(project(expression)));
            } else {
                out.push(JsxTypecheckEmit::Expression(project(expression)));
            }
        }
    }
}

fn project(expression: &JsxSyntaxExpression) -> JsxTypecheckExpression {
    JsxTypecheckExpression {
        code: expression.code.clone(),
        span: expression.span,
    }
}

fn push_expression(content: &str, start: u32, end: u32, out: &mut Vec<JsxTypecheckEmit>) {
    let content = content.trim();
    if !content.is_empty() {
        out.push(JsxTypecheckEmit::Expression(JsxTypecheckExpression {
            code: content.into(),
            span: JsxSyntaxSpan::new(start, end),
        }));
    }
}

#[cfg(test)]
pub(super) fn record_lowering() {
    counters::record_lowering();
}

#[cfg(test)]
pub(super) fn record_direct_fallback() {
    counters::record_direct_fallback();
}

#[cfg(test)]
pub(super) fn reset_lowering_counts() {
    counters::reset_lowering_counts();
}

#[cfg(test)]
pub(super) fn lowering_counts() -> (usize, usize) {
    counters::lowering_counts()
}

#[cfg(test)]
mod counters {
    use std::cell::Cell;

    thread_local! {
        static PROJECTIONS: Cell<usize> = const { Cell::new(0) };
        static DIRECT_FALLBACKS: Cell<usize> = const { Cell::new(0) };
    }

    pub(super) fn record_lowering() {
        PROJECTIONS.set(PROJECTIONS.get() + 1);
    }

    pub(super) fn record_direct_fallback() {
        DIRECT_FALLBACKS.set(DIRECT_FALLBACKS.get() + 1);
    }

    pub(super) fn reset_lowering_counts() {
        PROJECTIONS.set(0);
        DIRECT_FALLBACKS.set(0);
    }

    pub(super) fn lowering_counts() -> (usize, usize) {
        (PROJECTIONS.get(), DIRECT_FALLBACKS.get())
    }
}
