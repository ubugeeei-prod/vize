//! Shared helpers for the JSX lowering integration tests.
//!
//! Each integration test binary pulls in this module but uses only a subset of
//! the helpers, so unused-helper warnings are expected and silenced.
#![allow(dead_code)]

use vize_atelier_jsx::{JsxLang, LowerOutput, lower_source};
use vize_carton::Bump;
use vize_relief::{
    AttributeNode, DirectiveNode, ElementNode, ExpressionNode, PropNode, TemplateChildNode,
    TextNode,
};

/// Lower JSX source, asserting a single error-free render root, and return it.
pub fn lower_one<'a>(bump: &'a Bump, source: &str) -> vize_relief::RootNode<'a> {
    lower_one_in(bump, source, JsxLang::Jsx)
}

/// Lower TSX source, asserting a single error-free render root, and return it.
pub fn lower_one_tsx<'a>(bump: &'a Bump, source: &str) -> vize_relief::RootNode<'a> {
    lower_one_in(bump, source, JsxLang::Tsx)
}

fn lower_one_in<'a>(bump: &'a Bump, source: &str, lang: JsxLang) -> vize_relief::RootNode<'a> {
    lower_single(bump, source, lang).root
}

/// Lower source, asserting a single error-free render root, returning the full
/// [`LoweredRoot`] (including mode/name metadata).
pub fn lower_single<'a>(
    bump: &'a Bump,
    source: &str,
    lang: JsxLang,
) -> vize_atelier_jsx::LoweredRoot<'a> {
    let out = lower_source(bump, source, lang);
    assert!(
        !out.has_errors(),
        "unexpected diagnostics: {:?}",
        out.diagnostics
    );
    assert_eq!(out.roots.len(), 1, "expected exactly one render root");
    out.roots.into_iter().next().unwrap()
}

/// Lower JSX source and return the full output (roots + diagnostics).
pub fn lower_all<'a>(bump: &'a Bump, source: &str) -> LowerOutput<'a> {
    lower_source(bump, source, JsxLang::Jsx)
}

/// Borrow a child as an element, panicking otherwise.
pub fn as_element<'a>(child: &'a TemplateChildNode<'a>) -> &'a ElementNode<'a> {
    match child {
        TemplateChildNode::Element(element) => element,
        other => panic!("expected element child, got {:?}", other.node_type()),
    }
}

/// Borrow a child as a text node, panicking otherwise.
pub fn as_text<'a>(child: &'a TemplateChildNode<'a>) -> &'a TextNode {
    match child {
        TemplateChildNode::Text(text) => text,
        other => panic!("expected text child, got {:?}", other.node_type()),
    }
}

/// The first child of a root, as an element.
pub fn root_element<'a>(root: &'a vize_relief::RootNode<'a>) -> &'a ElementNode<'a> {
    as_element(&root.children[0])
}

/// Borrow a prop as a static attribute.
pub fn as_attribute<'a>(prop: &'a PropNode<'a>) -> &'a AttributeNode {
    match prop {
        PropNode::Attribute(attr) => attr,
        PropNode::Directive(dir) => panic!("expected attribute, got directive {:?}", dir.name),
    }
}

/// Borrow a prop as a directive.
pub fn as_directive<'a>(prop: &'a PropNode<'a>) -> &'a DirectiveNode<'a> {
    match prop {
        PropNode::Directive(dir) => dir,
        PropNode::Attribute(attr) => panic!("expected directive, got attribute {:?}", attr.name),
    }
}

/// The textual content of a simple expression.
pub fn simple_content<'a>(expr: &'a ExpressionNode<'a>) -> &'a str {
    match expr {
        ExpressionNode::Simple(simple) => simple.content.as_str(),
        ExpressionNode::Compound(_) => panic!("expected simple expression, got compound"),
    }
}

/// Whether a simple expression is marked static.
pub fn is_static(expr: &ExpressionNode<'_>) -> bool {
    match expr {
        ExpressionNode::Simple(simple) => simple.is_static,
        ExpressionNode::Compound(_) => panic!("expected simple expression, got compound"),
    }
}

/// Find the first directive on an element with the given normalized name.
pub fn find_directive<'a>(
    element: &'a ElementNode<'a>,
    name: &str,
) -> Option<&'a DirectiveNode<'a>> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == name => Some(&**dir),
        _ => None,
    })
}
