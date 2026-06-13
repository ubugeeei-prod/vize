//! Lowering JSX component-slot idioms (object / render-prop children) into
//! `<template v-slot>` synthetic elements, and the resulting VDOM codegen.
//!
//! The babel-plugin-jsx slot forms (`<Comp>{{ name: () => … }}</Comp>` and
//! `<List>{(p) => …}</List>`) are lowered into the same `<template v-slot>`
//! children the SFC path produces, so the shared slot transform + codegen build
//! a real `_withCtx` slots object — we are the parent *passing* slots, not the
//! component rendering them, so the output uses `_withCtx` (not `_renderSlot`).

mod common;

use vize_atelier_jsx::{JsxLang, lower_source};
use vize_carton::Bump;
use vize_relief::ElementType;
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

#[test]
fn slot_codegen_snapshot() {
    let cases = [
        (
            "object child named slots",
            "const A = () => <Comp>{{ header: () => <h1>Hi</h1>, footer: () => <p>Bye</p> }}</Comp>;",
        ),
        (
            "scoped named slot",
            "const A = () => <List>{{ item: ({ x }) => <li>{x}</li> }}</List>;",
        ),
        (
            "render prop default slot",
            "const A = () => <List>{(item) => <li>{item}</li>}</List>;",
        ),
        (
            "default and named slots",
            "const A = () => <Card>{{ default: () => <p>body</p>, title: () => <h1>T</h1> }}</Card>;",
        ),
        (
            "implicit default slot",
            "const A = () => <Card><h1>Title</h1></Card>;",
        ),
    ];

    insta::assert_snapshot!(common::snapshot_cases(&cases, |source| {
        common::vdom_code(source, JsxLang::Jsx)
    }));
}

// 6. IR-level: the lowered slot is a `<template>` with `tag_type == Template`
//    and a `slot` directive whose static arg is the slot name — exactly the
//    shape the Vapor slot-IR build keys off (Vapor codegen for JSX is not yet
//    wired in this crate, so we assert the slot KEY at the IR layer).
#[test]
fn slot_lowers_to_template_with_slot_directive() {
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const A = () => <Comp>{{ header: () => <h1>Hi</h1> }}</Comp>;",
        JsxLang::Jsx,
    );
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    let root = &out.roots[0].root;
    let TemplateChildNode::Element(comp) = &root.children[0] else {
        panic!("expected component element root");
    };
    assert_eq!(comp.tag.as_str(), "Comp");

    let TemplateChildNode::Element(template) = &comp.children[0] else {
        panic!("expected a synthetic <template> slot child");
    };
    assert_eq!(template.tag.as_str(), "template");
    assert_eq!(template.tag_type, ElementType::Template);

    let slot_dir = template
        .props
        .iter()
        .find_map(|prop| match prop {
            PropNode::Directive(dir) if dir.name.as_str() == "slot" => Some(&**dir),
            _ => None,
        })
        .expect("template carries a `slot` directive");

    // The slot KEY ("header") is the static directive arg.
    let arg = slot_dir.arg.as_ref().expect("slot directive has an arg");
    match arg {
        ExpressionNode::Simple(simple) => {
            assert!(simple.is_static, "slot name must be static");
            assert_eq!(simple.content.as_str(), "header");
        }
        ExpressionNode::Compound(_) => panic!("slot name should be a simple static expression"),
    }
    // Non-scoped slot has no params.
    assert!(slot_dir.exp.is_none(), "unscoped slot has no params");

    // The slot body is the lowered <h1>.
    let TemplateChildNode::Element(body) = &template.children[0] else {
        panic!("expected lowered slot body element");
    };
    assert_eq!(body.tag.as_str(), "h1");
}

// Scoped-slot params carry the RAW pattern source on the directive `exp`.
#[test]
fn scoped_slot_directive_carries_raw_param_pattern() {
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const A = () => <List>{{ item: ({ x }) => <li>{x}</li> }}</List>;",
        JsxLang::Jsx,
    );
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    let root = &out.roots[0].root;
    let TemplateChildNode::Element(list) = &root.children[0] else {
        panic!("expected component element root");
    };
    let TemplateChildNode::Element(template) = &list.children[0] else {
        panic!("expected synthetic <template> slot child");
    };
    let slot_dir = template
        .props
        .iter()
        .find_map(|prop| match prop {
            PropNode::Directive(dir) if dir.name.as_str() == "slot" => Some(&**dir),
            _ => None,
        })
        .expect("template carries a `slot` directive");
    let exp = slot_dir
        .exp
        .as_ref()
        .expect("scoped slot carries a param pattern");
    match exp {
        ExpressionNode::Simple(simple) => {
            assert!(!simple.is_static, "scoped params are dynamic");
            assert_eq!(simple.loc.source.as_str(), "{ x }");
        }
        ExpressionNode::Compound(_) => panic!("expected simple param expression"),
    }
}
