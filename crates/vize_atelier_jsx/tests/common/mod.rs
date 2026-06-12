//! Shared helpers for the JSX lowering integration tests.
//!
//! Each integration test binary pulls in this module but uses only a subset of
//! the helpers, so unused-helper warnings are expected and silenced.
#![allow(dead_code)]

use std::fmt::Write as _;
use vize_atelier_jsx::{
    DomCompileOptions, JsxLang, LowerOutput, VaporCompileOptions, compile_to_dom, compile_to_vapor,
    lower_source,
};
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

/// Compile one component to VDOM render code.
pub fn dom_code(source: &str, lang: JsxLang) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_dom(&bump, source, lang, DomCompileOptions::default());
    assert!(
        !out.has_errors(),
        "unexpected diagnostics: {:?}",
        out.diagnostics
    );
    assert_eq!(out.components.len(), 1, "expected exactly one component");
    out.components.into_iter().next().unwrap().code
}

/// Compile one component to Vapor render code.
pub fn vapor_code(source: &str, lang: JsxLang, ssr: bool) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_vapor(&bump, source, lang, VaporCompileOptions { ssr });
    assert!(
        !out.has_errors(),
        "unexpected diagnostics: {:?}",
        out.diagnostics
    );
    assert_eq!(out.components.len(), 1, "expected exactly one component");
    out.components.into_iter().next().unwrap().code
}

/// Normalize generated text before writing it into snapshot files.
pub fn snapshot_text(source: &str) -> std::string::String {
    let mut output = std::string::String::with_capacity(source.len());
    for (index, line) in source.split('\n').enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str(line.trim_end_matches(|ch| ch == ' ' || ch == '\t'));
    }
    output
}

/// Render a named case matrix into a single deterministic snapshot body.
pub fn snapshot_cases(
    cases: &[(&str, &str)],
    mut compile: impl FnMut(&str) -> vize_carton::String,
) -> std::string::String {
    let mut snapshot = std::string::String::new();
    for (index, (name, source)) in cases.iter().enumerate() {
        if index > 0 {
            snapshot.push_str("\n\n");
        }
        writeln!(snapshot, "## {name}").unwrap();
        writeln!(snapshot, "### source").unwrap();
        writeln!(snapshot, "{source}").unwrap();
        writeln!(snapshot, "### output").unwrap();
        snapshot.push_str(snapshot_text(compile(source).as_str()).as_str());
    }
    snapshot
}

/// Render a named, language-aware case matrix into a snapshot body.
pub fn snapshot_lang_cases(
    cases: &[(&str, JsxLang, &str)],
    mut compile: impl FnMut(&str, JsxLang) -> vize_carton::String,
) -> std::string::String {
    let mut snapshot = std::string::String::new();
    for (index, (name, lang, source)) in cases.iter().enumerate() {
        if index > 0 {
            snapshot.push_str("\n\n");
        }
        writeln!(snapshot, "## {name}").unwrap();
        writeln!(snapshot, "### lang").unwrap();
        writeln!(snapshot, "{lang:?}").unwrap();
        writeln!(snapshot, "### source").unwrap();
        writeln!(snapshot, "{source}").unwrap();
        writeln!(snapshot, "### output").unwrap();
        snapshot.push_str(snapshot_text(compile(source, *lang).as_str()).as_str());
    }
    snapshot
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
