//! Lowering of JSX children: text cleaning, interpolation, mixed content.

mod common;

use common::{as_text, lower_one, root_element, simple_content};
use vize_carton::Bump;
use vize_relief::TemplateChildNode;

#[test]
fn plain_text_child_is_lowered() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <p>Hello</p>;");
    let p = root_element(&root);
    assert_eq!(p.children.len(), 1);
    assert_eq!(as_text(&p.children[0]).content.as_str(), "Hello");
}

#[test]
fn whitespace_only_children_are_dropped() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <ul>\n   <li/>\n</ul>;");
    let ul = root_element(&root);
    // The surrounding newlines/indentation collapse to nothing, leaving one li.
    assert_eq!(ul.children.len(), 1);
}

#[test]
fn expression_child_becomes_interpolation() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <p>{count}</p>;");
    let p = root_element(&root);
    match &p.children[0] {
        TemplateChildNode::Interpolation(interp) => {
            assert_eq!(simple_content(&interp.content), "count");
        }
        other => panic!("expected interpolation, got {:?}", other.node_type()),
    }
}

#[test]
fn complex_expression_child_keeps_source_text() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <p>{a + b * c}</p>;");
    let p = root_element(&root);
    match &p.children[0] {
        TemplateChildNode::Interpolation(interp) => {
            assert_eq!(simple_content(&interp.content), "a + b * c");
        }
        other => panic!("expected interpolation, got {:?}", other.node_type()),
    }
}

#[test]
fn string_literal_container_becomes_text() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <p>{'literal'}</p>;");
    let p = root_element(&root);
    assert_eq!(as_text(&p.children[0]).content.as_str(), "literal");
}

#[test]
fn explicit_space_idiom_is_text() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <p>a{' '}b</p>;");
    let p = root_element(&root);
    let texts: Vec<&str> = p
        .children
        .iter()
        .filter_map(|c| match c {
            TemplateChildNode::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(texts, vec!["a", " ", "b"]);
}

#[test]
fn empty_expression_container_is_dropped() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <p>{/* a comment */}</p>;");
    let p = root_element(&root);
    assert_eq!(p.children.len(), 0);
}

#[test]
fn mixed_text_and_expression_children() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <p>Hi {name}!</p>;");
    let p = root_element(&root);
    // "Hi " text, {name} interpolation, "!" text.
    assert_eq!(p.children.len(), 3);
    assert_eq!(as_text(&p.children[0]).content.as_str(), "Hi ");
    assert!(matches!(
        &p.children[1],
        TemplateChildNode::Interpolation(_)
    ));
    assert_eq!(as_text(&p.children[2]).content.as_str(), "!");
}
