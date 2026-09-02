//! Lowering of components, multiple roots, and slot-shaped children.

mod common;

use common::{
    as_element, as_text, find_directive, is_static, lower_all, lower_one, root_element,
    simple_content,
};
use vize_relief::{ElementType, TemplateChildNode};
use vize_s0::Allocator;

#[test]
fn multiple_top_level_roots_are_each_lowered() {
    let bump = Allocator::new();
    let out = lower_all(&bump, "const A = () => <a/>;\nconst B = () => <b/>;");
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert_eq!(out.roots.len(), 2);
    assert_eq!(root_element(&out.roots[0].root).tag, "a");
    assert_eq!(root_element(&out.roots[1].root).tag, "b");
}

#[test]
fn component_with_element_children() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <Card><h1>Title</h1></Card>;");
    let card = root_element(&root);
    assert_eq!(card.tag, "Card");
    assert_eq!(card.tag_type, ElementType::Component);
    assert_eq!(as_element(&card.children[0]).tag, "h1");
}

#[test]
fn object_slot_children_become_slot_templates() {
    let bump = Allocator::new();
    // babel-plugin-jsx slot object syntax: the single object-expression child is
    // synthesized into `<template v-slot:name>` element children that the shared
    // slot transform + codegen turn into a real slots object.
    let root = lower_one(&bump, "const a = <Comp>{{ default: () => <p/> }}</Comp>;");
    let comp = root_element(&root);
    let template = as_element(&comp.children[0]);
    assert_eq!(template.tag, "template");
    assert_eq!(template.tag_type, ElementType::Template);
    let slot = find_directive(template, "slot").expect("template carries a `slot` directive");
    let arg = slot
        .arg
        .as_ref()
        .expect("slot directive has a static name arg");
    assert!(is_static(arg), "slot name is static");
    assert_eq!(as_element(&template.children[0]).tag, "p");
}

#[test]
fn render_prop_child_becomes_default_slot_template() {
    let bump = Allocator::new();
    // A single render-prop child becomes a scoped default slot template.
    let root = lower_one(&bump, "const a = <List>{(item) => <li/>}</List>;");
    let list = root_element(&root);
    let template = as_element(&list.children[0]);
    assert_eq!(template.tag, "template");
    assert_eq!(template.tag_type, ElementType::Template);
    let slot = find_directive(template, "slot").expect("template carries a `slot` directive");
    assert!(slot.exp.is_some(), "scoped slot carries the param pattern");
    assert_eq!(as_element(&template.children[0]).tag, "li");
}

#[test]
fn nested_components_and_intrinsics_mix() {
    let bump = Allocator::new();
    let root = lower_one(
        &bump,
        "const a = <Layout><Header/><main><Content/></main></Layout>;",
    );
    let layout = root_element(&root);
    assert_eq!(layout.children.len(), 2);
    assert_eq!(
        as_element(&layout.children[0]).tag_type,
        ElementType::Component
    );
    let main = as_element(&layout.children[1]);
    assert_eq!(main.tag_type, ElementType::Element);
    assert_eq!(as_element(&main.children[0]).tag, "Content");
}

#[test]
fn jsx_in_return_statement_is_found() {
    let bump = Allocator::new();
    let out = lower_all(&bump, "function App() {\n  return <div>ok</div>;\n}");
    assert_eq!(out.roots.len(), 1);
    assert_eq!(root_element(&out.roots[0].root).tag, "div");
}

#[test]
fn jsx_in_ternary_finds_both_branches() {
    let bump = Allocator::new();
    let out = lower_all(&bump, "const a = ok ? <yes/> : <no/>;");
    assert_eq!(out.roots.len(), 2);
}

#[test]
fn render_prop_child_with_a_plain_body_keeps_its_value() {
    let bump = Allocator::new();
    // `<B>{() => 'foo'}</B>` used to produce an empty default slot: the slot
    // body was only lowered for JSX and control-flow shapes, and anything else
    // was dropped (#3421).
    let root = lower_one(&bump, "const a = <B>{() => 'foo'}</B>;");
    let component = root_element(&root);
    let template = as_element(&component.children[0]);
    assert_eq!(template.tag, "template");
    assert_eq!(template.children.len(), 1);
    assert_eq!(as_text(&template.children[0]).content, "foo");
}

#[test]
fn render_prop_child_with_an_expression_body_keeps_its_value() {
    let bump = Allocator::new();
    let root = lower_one(&bump, "const a = <B>{() => label}</B>;");
    let component = root_element(&root);
    let template = as_element(&component.children[0]);
    let TemplateChildNode::Interpolation(interpolation) = &template.children[0] else {
        panic!("expected an interpolation slot body");
    };
    assert_eq!(simple_content(&interpolation.content), "label");
}
