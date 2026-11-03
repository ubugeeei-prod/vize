//! Vue component nodes retain their authored table scope (#11728).

use super::{Parser, parse, parse_with_options};
use vize_l0::Allocator;
use vize_relief::{ElementType, ParserOptions, TemplateChildNode};

#[test]
fn dynamic_component_stays_inside_its_authored_table_loop() {
    let allocator = Allocator::new();
    let source = r#"<table><tr v-for="(line, r) in rows"><component :is="r === 0 ? 'th' : 'td'" v-for="(cell, c) in line">{{ cell }}</component></tr></table>"#;
    let (root, errors) = parse(&allocator, source);
    assert_eq!(root.children.len(), 1, "{errors:?}");
    let TemplateChildNode::Element(table) = &root.children[0] else {
        panic!("authored table");
    };
    assert_eq!(table.tag, "table");
    let TemplateChildNode::Element(section) = &table.children[0] else {
        panic!("implicit table section");
    };
    let TemplateChildNode::Element(row) = &section.children[0] else {
        panic!("authored row");
    };
    assert_eq!(row.tag, "tr");
    let TemplateChildNode::Element(component) = &row.children[0] else {
        panic!("authored component");
    };
    assert_eq!(component.tag, "component");
    assert_eq!(component.tag_type, ElementType::Component);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn native_table_recovery_still_fosters_invalid_html_children() {
    let allocator = Allocator::new();
    let (root, _) = parse(
        &allocator,
        "<table><tr><div>invalid HTML</div></tr></table>",
    );
    assert_eq!(root.children.len(), 2);
    let TemplateChildNode::Element(div) = &root.children[0] else {
        panic!("fostered native element");
    };
    assert_eq!(div.tag, "div");
    let TemplateChildNode::Element(table) = &root.children[1] else {
        panic!("original table");
    };
    assert_eq!(table.tag, "table");
}

#[test]
fn frozen_component_remains_a_literal_html_element() {
    let allocator = Allocator::new();
    let (root, _, _) = Parser::new(
        &allocator,
        "<table v-pre><tr><component>literal</component></tr></table>",
    )
    .parse_with_frozen_elements();
    assert_eq!(root.children.len(), 2);
    let TemplateChildNode::Element(component) = &root.children[0] else {
        panic!("literal component");
    };
    assert_eq!(component.tag, "component");
    assert_eq!(component.tag_type, ElementType::Element);
}

#[test]
fn configured_custom_element_keeps_its_native_table_policy() {
    let allocator = Allocator::new();
    let options = ParserOptions {
        is_custom_element: Some(|tag| tag == "component"),
        ..ParserOptions::default()
    };
    let (root, _) = parse_with_options(
        &allocator,
        "<table><tr><component>custom</component></tr></table>",
        options,
    );
    assert_eq!(root.children.len(), 2);
    let TemplateChildNode::Element(component) = &root.children[0] else {
        panic!("configured custom element");
    };
    assert_eq!(component.tag_type, ElementType::Element);
}
