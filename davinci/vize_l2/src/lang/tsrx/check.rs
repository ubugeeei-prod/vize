//! Every unsupported construct is found before any canonical op is written.

use alloc::vec::Vec;
use vize_l0::Span;
use vize_l1::markup::tsrx::{
    Attribute, AttributeValue, Block, Child, Element, Node, Region, RegionSite, TagName,
};

use super::{TsrxIssue, TsrxIssueKind};

pub(super) fn region(source: &str, region: &Region<'_>, issues: &mut Vec<TsrxIssue>) {
    match (&region.node, region.site) {
        // A function body's setup statements are host code; only its render
        // node is markup.
        (Node::CodeBlock(block), RegionSite::FunctionBody) => {
            if let Some(render) = &block.render {
                node(source, render, issues);
            }
        }
        (node_, _) => node(source, node_, issues),
    }
}

fn node(source: &str, node_: &Node<'_>, issues: &mut Vec<TsrxIssue>) {
    let (kind, span) = match node_ {
        Node::Element(element_) => return element(source, element_, issues),
        Node::Fragment(fragment) => return children(source, &fragment.children, issues),
        Node::CodeBlock(block_) => return block(source, block_, issues),
        Node::If(conditional) => {
            for branch in &conditional.branches {
                block(source, &branch.body, issues);
            }
            return;
        }
        Node::For(iteration) => (TsrxIssueKind::For, iteration.span),
        Node::Switch(switch) => (TsrxIssueKind::Switch, switch.span),
        Node::Try(boundary) => (TsrxIssueKind::Try, boundary.span),
    };
    issues.push(TsrxIssue { kind, span });
}

fn block(source: &str, block: &Block<'_>, issues: &mut Vec<TsrxIssue>) {
    if !block.statements.is_empty() {
        issues.push(TsrxIssue {
            kind: TsrxIssueKind::StatementContainer,
            span: block.span,
        });
    }
    if let Some(render) = &block.render {
        node(source, render, issues);
    }
}

fn element(source: &str, element: &Element<'_>, issues: &mut Vec<TsrxIssue>) {
    if let TagName::Dynamic { span, .. } = element.name {
        issues.push(TsrxIssue {
            kind: TsrxIssueKind::DynamicTag,
            span,
        });
    }
    // Style and script elements need the TSRX style and script profile,
    // whichever body form the grammar recorded.
    if element.raw_text.is_some()
        || matches!(element.name, TagName::Static(span) if matches!(slice(source, span), "style" | "script"))
    {
        issues.push(TsrxIssue {
            kind: TsrxIssueKind::RawText,
            span: element.span,
        });
    }
    for attribute in &element.attributes {
        if let Attribute::Named {
            value: Some(AttributeValue::String(span)),
            ..
        } = attribute
        {
            let value = slice(source, *span);
            if value.contains('&') {
                issues.push(TsrxIssue {
                    kind: TsrxIssueKind::Entity,
                    span: *span,
                });
            }
            if value.contains(['\n', '\r']) {
                issues.push(TsrxIssue {
                    kind: TsrxIssueKind::AttributeLineBreak,
                    span: *span,
                });
            }
        }
    }
    children(source, &element.children, issues);
}

fn children(source: &str, children: &[Child<'_>], issues: &mut Vec<TsrxIssue>) {
    for child in children {
        match child {
            Child::Text(span) if slice(source, *span).contains('&') => issues.push(TsrxIssue {
                kind: TsrxIssueKind::Entity,
                span: *span,
            }),
            Child::Node(child) => node(source, child, issues),
            Child::Text(_) | Child::Comment(_) | Child::Expression(_) => {}
        }
    }
}

pub(super) fn slice(source: &str, span: Span) -> &str {
    source
        .get(span.start as usize..span.end as usize)
        .unwrap_or_default()
}
