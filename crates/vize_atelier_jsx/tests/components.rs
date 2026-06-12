//! Lowering of components, multiple roots, and slot-shaped children.

mod common;

use common::{as_element, find_directive, is_static, lower_all, lower_one, root_element};
use vize_carton::Bump;
use vize_relief::ast::core::ElementType;

#[test]
fn multiple_top_level_roots_are_each_lowered() {
    let bump = Bump::new();
    let out = lower_all(&bump, "const A = () => <a/>;\nconst B = () => <b/>;");
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert_eq!(out.roots.len(), 2);
    assert_eq!(root_element(&out.roots[0].root).tag.as_str(), "a");
    assert_eq!(root_element(&out.roots[1].root).tag.as_str(), "b");
}

#[test]
fn component_with_element_children() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <Card><h1>Title</h1></Card>;");
    let card = root_element(&root);
    assert_eq!(card.tag.as_str(), "Card");
    assert_eq!(card.tag_type, ElementType::Component);
    assert_eq!(as_element(&card.children[0]).tag.as_str(), "h1");
}

#[test]
fn object_slot_children_become_slot_templates() {
    let bump = Bump::new();
    // babel-plugin-jsx slot object syntax: the single object-expression child is
    // synthesized into `<template v-slot:name>` element children that the shared
    // slot transform + codegen turn into a real slots object.
    let root = lower_one(&bump, "const a = <Comp>{{ default: () => <p/> }}</Comp>;");
    let comp = root_element(&root);
    let template = as_element(&comp.children[0]);
    assert_eq!(template.tag.as_str(), "template");
    assert_eq!(template.tag_type, ElementType::Template);
    let slot = find_directive(template, "slot").expect("template carries a `slot` directive");
    let arg = slot
        .arg
        .as_ref()
        .expect("slot directive has a static name arg");
    assert!(is_static(arg), "slot name is static");
    assert_eq!(as_element(&template.children[0]).tag.as_str(), "p");
}

#[test]
fn render_prop_child_becomes_default_slot_template() {
    let bump = Bump::new();
    // A single render-prop child becomes a scoped default slot template.
    let root = lower_one(&bump, "const a = <List>{(item) => <li/>}</List>;");
    let list = root_element(&root);
    let template = as_element(&list.children[0]);
    assert_eq!(template.tag.as_str(), "template");
    assert_eq!(template.tag_type, ElementType::Template);
    let slot = find_directive(template, "slot").expect("template carries a `slot` directive");
    assert!(slot.exp.is_some(), "scoped slot carries the param pattern");
    assert_eq!(as_element(&template.children[0]).tag.as_str(), "li");
}

#[test]
fn nested_components_and_intrinsics_mix() {
    let bump = Bump::new();
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
    assert_eq!(as_element(&main.children[0]).tag.as_str(), "Content");
}

#[test]
fn jsx_in_return_statement_is_found() {
    let bump = Bump::new();
    let out = lower_all(&bump, "function App() {\n  return <div>ok</div>;\n}");
    assert_eq!(out.roots.len(), 1);
    assert_eq!(root_element(&out.roots[0].root).tag.as_str(), "div");
}

#[test]
fn jsx_in_ternary_finds_both_branches() {
    let bump = Bump::new();
    let out = lower_all(&bump, "const a = ok ? <yes/> : <no/>;");
    assert_eq!(out.roots.len(), 2);
}
