//! Lowering JSX component-slot idioms (object / render-prop children) into
//! `<template v-slot>` synthetic elements, and the resulting VDOM codegen.
//!
//! The babel-plugin-jsx slot forms (`<Comp>{{ name: () => … }}</Comp>` and
//! `<List>{(p) => …}</List>`) are lowered into the same `<template v-slot>`
//! children the SFC path produces, so the shared slot transform + codegen build
//! a real `_withCtx` slots object — we are the parent *passing* slots, not the
//! component rendering them, so the output uses `_withCtx` (not `_renderSlot`).

use vize_atelier_jsx::{DomCompileOptions, JsxLang, compile_to_dom, lower_source};
use vize_carton::Bump;
use vize_relief::ElementType;
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

/// Compile JSX to VDOM render code, asserting a single error-free component.
fn dom(src: &str) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_dom(&bump, src, JsxLang::Jsx, DomCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

// 1. Object child => named slots, with a stable `_: 1` flag.
#[test]
fn object_child_lowers_to_named_slots() {
    let code = dom(
        "const A = () => <Comp>{{ header: () => <h1>Hi</h1>, footer: () => <p>Bye</p> }}</Comp>;",
    );
    assert!(code.contains("_withCtx"), "{code}");
    assert!(code.contains("header:"), "{code}");
    assert!(code.contains("footer:"), "{code}");
    assert!(code.contains("_createElementVNode(\"h1\""), "{code}");
    assert!(code.contains("_: 1 /* STABLE */"), "{code}");
}

// 2. Scoped named slot: destructured param stays bare (no `_ctx.`).
#[test]
fn object_child_supports_scoped_slot_params() {
    let code = dom("const A = () => <List>{{ item: ({ x }) => <li>{x}</li> }}</List>;");
    assert!(code.contains("item: _withCtx(({ x }) =>"), "{code}");
    assert!(code.contains("_toDisplayString(x)"), "{code}");
    assert!(!code.contains("_ctx.x"), "{code}");
}

// 3. Single render-prop child => default scoped slot, bare param.
#[test]
fn render_prop_child_lowers_to_default_scoped_slot() {
    let code = dom("const A = () => <List>{(item) => <li>{item}</li>}</List>;");
    assert!(code.contains("default: _withCtx((item) =>"), "{code}");
    assert!(code.contains("_toDisplayString(item)"), "{code}");
    assert!(!code.contains("_ctx.item"), "{code}");
}

// 4. Object child mixing `default` with a named slot.
#[test]
fn object_child_mixes_default_and_named_slots() {
    let code = dom(
        "const A = () => <Card>{{ default: () => <p>body</p>, title: () => <h1>T</h1> }}</Card>;",
    );
    assert!(code.contains("default: _withCtx"), "{code}");
    assert!(code.contains("title: _withCtx"), "{code}");
}

// 5. Regression: plain element children still form an implicit default slot.
#[test]
fn plain_element_children_stay_implicit_default_slot() {
    let code = dom("const A = () => <Card><h1>Title</h1></Card>;");
    assert!(code.contains("default: _withCtx(() =>"), "{code}");
    assert!(code.contains("_createElementVNode(\"h1\""), "{code}");
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
