//! Canonical construction for checked regions, in authored preorder.

use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};
use vize_l1::markup::tsrx::{
    Attribute, AttributeValue, Block, Child, Element, For, Node, Region, RegionSite, TagName,
};

use super::{TsrxIssue, TsrxIssueKind, expression, is_component, text};
use crate::artifact::{ArtifactError, HostForHead, OpFamily, RegionBuilder};
use crate::expr::ExprRef;
use crate::op::{self, Namespace};

/// The component a dynamic tag renders through.
const DYNAMIC_COMPONENT: &str = "component";

pub(super) struct Context<'a> {
    pub allocator: &'a Allocator,
    pub source: &'a str,
}

type Result<T = ()> = core::result::Result<T, TsrxIssue>;

pub(super) fn region<'a>(
    cx: &Context<'a>,
    region: &'a Region<'a>,
    builder: &mut RegionBuilder<'_, 'a>,
) -> Result {
    match (&region.node, region.site) {
        (Node::CodeBlock(block), RegionSite::FunctionBody) => match &block.render {
            Some(render) => node(cx, render, Namespace::Html, builder),
            None => Ok(()),
        },
        (root, _) => node(cx, root, Namespace::Html, builder),
    }
}

fn node<'a>(
    cx: &Context<'a>,
    node_: &'a Node<'a>,
    namespace: Namespace,
    builder: &mut RegionBuilder<'_, 'a>,
) -> Result {
    match node_ {
        Node::Element(element_) => element(cx, element_, namespace, builder),
        Node::Fragment(fragment) => children(cx, &fragment.children, namespace, builder),
        Node::CodeBlock(block_) => block(cx, block_, namespace, builder),
        Node::If(conditional) => {
            let mut failure = None;
            builder
                .conditional(conditional.span, |branches, _| {
                    for branch in &conditional.branches {
                        let condition = match &branch.test {
                            Some(test) => match expression(cx.allocator, cx.source, test) {
                                Ok(test) => Some(ExprRef::Js(test)),
                                Err(issue) => {
                                    failure = Some(issue);
                                    return;
                                }
                            },
                            None => None,
                        };
                        let mut inner = None;
                        let built = branches.branch(condition, branch.span, |region, _| {
                            inner = block(cx, &branch.body, namespace, region).err();
                        });
                        if let Some(issue) = inner.or(artifact(built.err(), branch.span)) {
                            failure = Some(issue);
                            return;
                        }
                    }
                })
                .map_err(|error| issue(TsrxIssueKind::Artifact(error), conditional.span))?;
            failure.map_or(Ok(()), Err)
        }
        Node::For(each) => host_for(cx, each, namespace, builder),
        // Refused by the check pass before construction starts.
        Node::Switch(_) | Node::Try(_) => Err(unchecked(node_.span())),
    }
}

/// `@for (left of right; index i; key k) { … } @empty { … }`: a `ui.for` with
/// the retained host head, whose `@empty` block is the loop's empty region.
fn host_for<'a>(
    cx: &Context<'a>,
    each: &'a For<'a>,
    namespace: Namespace,
    builder: &mut RegionBuilder<'_, 'a>,
) -> Result {
    let collection = expression(cx.allocator, cx.source, &each.right)?;
    let key = match &each.key {
        Some(key) => Some(ExprRef::Js(expression(cx.allocator, cx.source, key)?)),
        None => None,
    };
    let left = each.left.span();
    let head = HostForHead {
        left: &each.left,
        left_text: slice(cx, Span::new(left.start, left.end))?,
        collection,
        index: each.index.as_ref(),
        key,
        r#await: each.r#await,
    };
    let mut failure = None;
    builder
        .host_for(head, each.span, |regions, _| {
            let built = regions.body(|region, _| {
                failure = block(cx, &each.body, namespace, region).err();
            });
            failure = failure.or(artifact(built.err(), each.body.span));
            if let (None, Some(empty)) = (&failure, &each.empty) {
                let built = regions.empty(|region, _| {
                    failure = block(cx, empty, namespace, region).err();
                });
                failure = failure.or(artifact(built.err(), empty.span));
            }
        })
        .map_err(|error| issue(TsrxIssueKind::Artifact(error), each.span))?;
    failure.map_or(Ok(()), Err)
}

fn block<'a>(
    cx: &Context<'a>,
    block: &'a Block<'a>,
    namespace: Namespace,
    builder: &mut RegionBuilder<'_, 'a>,
) -> Result {
    if !block.statements.is_empty() {
        return Err(unchecked(block.span));
    }
    match &block.render {
        Some(render) => node(cx, render, namespace, builder),
        None => Ok(()),
    }
}

fn element<'a>(
    cx: &Context<'a>,
    element: &'a Element<'a>,
    parent: Namespace,
    builder: &mut RegionBuilder<'_, 'a>,
) -> Result {
    if element.raw_text.is_some() {
        return Err(unchecked(element.span));
    }
    let (name, tag) = match &element.name {
        TagName::Static(span) => (slice(cx, *span)?, None),
        TagName::Dynamic { span, expression } => (DYNAMIC_COMPONENT, Some((*span, expression))),
    };
    let mut attributes = vize_l0::Vec::new_in(&cx.allocator);
    for attribute in &element.attributes {
        if let Attribute::Named { span, name, value } = attribute {
            let value = match value {
                None => None,
                Some(AttributeValue::String(quoted)) => Some(slice(
                    cx,
                    Span::new(quoted.start + 1, quoted.end.saturating_sub(1)),
                )?),
                Some(AttributeValue::Expression { .. }) => continue,
            };
            attributes.push(op::Attribute {
                name: slice(cx, *name)?,
                value,
                span: *span,
            });
        }
    }
    let mut failure = None;
    let mut body = |region: &mut RegionBuilder<'_, 'a>, namespace: Namespace| {
        failure = dynamic_tag(cx, tag, region)
            .and_then(|()| bindings(cx, &element.attributes, region))
            .and_then(|()| children(cx, &element.children, namespace, region))
            .err();
    };
    let built = if tag.is_some() || is_component(name) {
        // Its children are its default slot, which the slot grouping pass
        // builds only when the lowering reports a slot carrier.
        builder.observe(OpFamily::SlotCarrier);
        builder.component(name, attributes, element.span, |region, _| {
            body(region, parent);
        })
    } else {
        let namespace = match (parent, name) {
            (_, "svg") => Namespace::Svg,
            (_, "math") => Namespace::MathMl,
            (other, _) => other,
        };
        let inner = match (namespace, name) {
            (Namespace::Svg, "foreignObject") => Namespace::Html,
            (other, _) => other,
        };
        builder.element(name, namespace, attributes, element.span, |region, _| {
            body(region, inner);
        })
    };
    failure
        .or(artifact(built.err(), element.span))
        .map_or(Ok(()), Err)
}

/// The tag of `<{expr}>`, bound as `is` at `{expr}`, before the attributes.
fn dynamic_tag<'a>(
    cx: &Context<'a>,
    tag: Option<(Span, &'a oxc_ast::ast::Expression<'a>)>,
    region: &mut RegionBuilder<'_, 'a>,
) -> Result {
    let Some((span, value)) = tag else {
        return Ok(());
    };
    let value = ExprRef::Js(expression(cx.allocator, cx.source, value)?);
    artifact(region.synthesized_bind("is", value, span).err(), span).map_or(Ok(()), Err)
}

fn bindings<'a>(
    cx: &Context<'a>,
    attributes: &'a [Attribute<'a>],
    region: &mut RegionBuilder<'_, 'a>,
) -> Result {
    for attribute in attributes {
        let (built, span) = match attribute {
            Attribute::Named {
                span,
                name,
                value:
                    Some(AttributeValue::Expression {
                        expression: value, ..
                    }),
            } => {
                let value = ExprRef::Js(expression(cx.allocator, cx.source, value)?);
                (region.bind(slice(cx, *name)?, *name, value, *span), *span)
            }
            Attribute::Named { .. } => continue,
            Attribute::Shorthand {
                span,
                expression: value,
            } => {
                let js = expression(cx.allocator, cx.source, value)?;
                (
                    region.bind(js.source, js.span, ExprRef::Js(js), *span),
                    *span,
                )
            }
            Attribute::Spread {
                span,
                expression: value,
            } => {
                let value = ExprRef::Js(expression(cx.allocator, cx.source, value)?);
                (region.spread(value, *span), *span)
            }
        };
        artifact(built.err(), span).map_or(Ok(()), Err)?;
    }
    Ok(())
}

fn children<'a>(
    cx: &Context<'a>,
    children: &'a [Child<'a>],
    namespace: Namespace,
    region: &mut RegionBuilder<'_, 'a>,
) -> Result {
    for child in children {
        match child {
            Child::Text(span) => {
                let raw = slice(cx, *span)?;
                let content = match text::normalized(raw) {
                    Some(rendered) => cx.allocator.alloc_str(&rendered),
                    None => raw,
                };
                if !content.is_empty() {
                    artifact(region.text(content, *span).err(), *span).map_or(Ok(()), Err)?;
                }
            }
            // Comments between children are host comments, not rendered nodes.
            Child::Comment(_) => {}
            Child::Expression(child) => {
                if let Some(value) = &child.expression {
                    let value = ExprRef::Js(expression(cx.allocator, cx.source, value)?);
                    artifact(region.child_value(value, child.span).err(), child.span)
                        .map_or(Ok(()), Err)?;
                }
            }
            Child::Node(child) => node(cx, child, namespace, region)?,
        }
    }
    Ok(())
}

fn slice<'a>(cx: &Context<'a>, span: Span) -> Result<&'a str> {
    cx.source
        .get(span.start as usize..span.end as usize)
        .ok_or_else(|| {
            issue(
                TsrxIssueKind::Artifact(ArtifactError::InvalidSpan { node: None, span }),
                span,
            )
        })
}

fn artifact(error: Option<ArtifactError>, span: Span) -> Option<TsrxIssue> {
    error.map(|error| issue(TsrxIssueKind::Artifact(error), span))
}

/// A construct the check pass refuses reached construction: a compiler bug.
fn unchecked(span: Span) -> TsrxIssue {
    issue(
        TsrxIssueKind::Artifact(ArtifactError::InvalidSpan { node: None, span }),
        span,
    )
}

const fn issue(kind: TsrxIssueKind, span: Span) -> TsrxIssue {
    TsrxIssue { kind, span }
}
