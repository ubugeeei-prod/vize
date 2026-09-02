//! Forwarding an opaque slots object from `v-slots={slots}` (#3467).
//!
//! `v-slots={{ … }}` expands into synthetic `<template v-slot:name>` children
//! (#3418, `v_slots.rs`). `v-slots={slots}` cannot: the value only exists at
//! runtime, so there are no entries to expand. It lowers to a relief `slots`
//! directive that `vize_atelier_core`'s slot codegen emits as a spread.
//!
//! Both babel shapes are asserted on the **complete** generated render function,
//! against the recorded `@vue/babel-plugin-jsx` 2.0.1 output in the corpus rows
//! `slots/v_slots_only`, `slots/v_slots_with_children` and
//! `optimize/v_slots_stability`:
//!
//! ```js
//! const A = () => _createVNode(_resolveComponent("B"), null, slots);
//! const C = () => _createVNode(_resolveComponent("B"), null, {
//!   default: () => [_createVNode("div", null, [_createTextVNode("A")])],
//!   ...slots
//! });
//! ```
//!
//! # Why no `_` flag and why `1024 /* DYNAMIC_SLOTS */`
//!
//! Babel emits no slot-stability flag beside a forwarded object, even under
//! `optimize: true`, and neither does Vize. Only the no-`_` path runs Vue's
//! `normalizeObjectSlots`, which binds raw entries to the owning instance and
//! passes already-`withCtx`-wrapped ones through untouched via `rawSlot._n`.
//! `_: 2 /* DYNAMIC */` would `extend` without normalizing; `_: 1 /* STABLE */`
//! would stop the child re-rendering when the forwarded slots change. Babel's
//! unoptimized vnodes get that update for free (`shouldUpdateComponent` forces
//! it for any children); Vize's are always optimized, so the vnode carries
//! `1024 /* DYNAMIC_SLOTS */` to force it.

mod common;

use vize_atelier_jsx::{
    JsxLang, SsrCompileOptions, VaporCompileOptions, VdomCompileOptions, compile_to_ssr,
    compile_to_vapor, compile_to_vdom,
};
use vize_s0::Allocator;

fn render_code(source: &str) -> String {
    let bump = Allocator::new();
    let out = compile_to_vdom(&bump, source, JsxLang::Jsx, VdomCompileOptions::default());
    assert_eq!(
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str().to_string())
            .collect::<Vec<_>>(),
        Vec::<String>::new(),
        "forwarding a slots object to VDOM is supported and must not diagnose"
    );
    out.components
        .into_iter()
        .map(|component| component.code.as_str().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_forwarded_slots_object_is_the_children_argument() {
    // Corpus rows `slots/v_slots_only` and `optimize/v_slots_stability`: babel
    // emits `_createVNode(_resolveComponent("B"), null, slots)` for both, with
    // no `_` flag under either option set.
    assert_eq!(
        render_code("const A = () => <B v-slots={slots}/>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, slots, \
         1024 /* DYNAMIC_SLOTS */))\n}"
    );
}

#[test]
fn element_children_become_the_default_slot_and_the_spread_closes_the_object() {
    // Corpus row `slots/v_slots_with_children`: babel emits
    // `{default: () => [...], ...slots}` — authored slots first, spread last, so
    // a forwarded entry overrides an authored one of the same name.
    assert_eq!(
        render_code("const A = () => <B v-slots={slots}><div>A</div></B>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, {\n    \
         default: _withCtx(() => [\n      \
         _createElementVNode(\"div\", null, \"A\")\n    \
         ]),\n    \
         ...slots\n  \
         }, 1024 /* DYNAMIC_SLOTS */))\n}"
    );
}

#[test]
fn named_slot_templates_keep_the_spread_last() {
    assert_eq!(
        render_code("const A = () => <B v-slots={slots}><template v-slot:foo>x</template></B>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, {\n    \
         foo: _withCtx(() => [\n      \
         _createTextVNode(\"x\")\n    \
         ]),\n    \
         ...slots\n  \
         }, 1024 /* DYNAMIC_SLOTS */))\n}"
    );
}

#[test]
fn whitespace_only_children_do_not_build_an_object() {
    // Whitespace is not a default slot, so this is still the bare-value shape.
    assert_eq!(
        render_code("const A = () => <B v-slots={slots}>{' '}</B>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createBlock(_component_B, null, slots, \
         1024 /* DYNAMIC_SLOTS */))\n}"
    );
}

#[test]
fn any_opaque_expression_is_forwarded_verbatim() {
    // Parentheses and TS wrappers are transparent; what is emitted is the
    // expression underneath, so no TS-only syntax reaches the JavaScript.
    for (source, forwarded) in [
        ("<B v-slots={p.slots}/>", "p.slots"),
        ("<B v-slots={(slots)}/>", "slots"),
        ("<B v-slots={getSlots()}/>", "getSlots()"),
        ("<B v-slots={c ? a : b}/>", "c ? a : b"),
    ] {
        assert_eq!(
            render_code(&format!("const A = () => {source};")),
            format!(
                "export function render(_ctx, _cache) {{\n  \
                 const _component_B = _resolveComponent(\"B\")\n  \n  \
                 return (_openBlock(), _createBlock(_component_B, null, {forwarded}, \
                 1024 /* DYNAMIC_SLOTS */))\n}}"
            ),
            "{source}"
        );
    }
}

#[test]
fn a_forwarded_slots_object_survives_v_for() {
    // A component with no children of its own still needs the slots argument
    // emitted inside `renderList`, where the children slot is otherwise skipped.
    assert_eq!(
        render_code("const A = () => <ul>{items.map(i => <B key={i} v-slots={slots}/>)}</ul>;"),
        "export function render(_ctx, _cache) {\n  \
         const _component_B = _resolveComponent(\"B\")\n  \n  \
         return (_openBlock(), _createElementBlock(\"ul\", null, [\n    \
         (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (i) => {\n      \
         return (_openBlock(), _createBlock(_component_B, {\n        \
         key: i\n      \
         }, slots, 1024 /* DYNAMIC_SLOTS */))\n    \
         }), 128 /* KEYED_FRAGMENT */))\n  \
         ]))\n}"
    );
}

#[test]
fn v_slots_never_becomes_a_resolved_directive() {
    // The #3418 regression: an unrecognized `v-*` fell through to the custom
    // directive path. `slots` is a compiler built-in, so it contributes no prop,
    // no `withDirectives` wrapper and no `resolveDirective("slots")`.
    let bump = Allocator::new();
    let root = common::lower_one(&bump, "const A = () => <B class=\"c\" v-slots={slots}/>;");
    let element = common::root_element(&root);
    assert_eq!(element.props.len(), 2);
    assert_eq!(common::as_attribute(&element.props[0]).name, "class");
    let forwarded =
        common::find_directive(element, "slots").expect("v-slots lowers to a directive");
    assert!(forwarded.arg.is_none());
    let exp = forwarded.exp.as_ref().expect("forwarded expression");
    assert_eq!(common::simple_content(exp), "slots");
    assert!(!common::is_static(exp));
}

#[test]
fn vapor_output_reports_the_gap_instead_of_dropping_the_slots() {
    // Vapor builds its slots from the component's children and has no spread, so
    // it must not silently render a component with no slots.
    let bump = Allocator::new();
    let out = compile_to_vapor(
        &bump,
        "const A = () => <B v-slots={slots}/>;",
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
    assert_eq!(
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str().to_string())
            .collect::<Vec<_>>(),
        vec![
            "v-slots forwards a slots object the compiler cannot see inside, which Vapor \
             output cannot express: Vapor slots are built from the component's children. \
             Write the slots inline, e.g. v-slots={{ default: () => <div/> }}, or compile \
             this component to VDOM."
                .to_string()
        ]
    );
}

#[test]
fn ssr_output_reports_the_gap_instead_of_dropping_the_slots() {
    let bump = Allocator::new();
    let out = compile_to_ssr(
        &bump,
        "const A = () => <B v-slots={slots}/>;",
        JsxLang::Jsx,
        SsrCompileOptions::default(),
    );
    assert_eq!(
        out.diagnostics
            .iter()
            .map(|d| d.message.as_str().to_string())
            .collect::<Vec<_>>(),
        vec![
            "v-slots forwards a slots object the compiler cannot see inside, which SSR \
             output cannot express: the server renderer inlines each slot's content. \
             Write the slots inline, e.g. v-slots={{ default: () => <div/> }}, or render \
             this component on the client."
                .to_string()
        ]
    );
}
