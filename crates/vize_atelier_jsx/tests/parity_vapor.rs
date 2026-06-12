//! Vapor-backend JSX/TSX parity suite (Part of #1491).
//!
//! These mirror the reference areas of `vue-jsx-vapor` (compiler-rs) but assert
//! **Vize's** Vapor codegen output — `_template` strings, `_renderEffect` /
//! `_setProp` / `_setText` effects, `_createIf` / `_createFor` blocks, slot
//! objects on `_createComponentWithFallback` — rather than byte-for-byte parity,
//! since Vize emits through its own `vize_atelier_vapor` codegen path.
//!
//! Backend separation: every failure here points at the **Vapor** lowering +
//! codegen path. The VDOM mirror lives in `parity_vdom.rs`. See
//! `PARITY_INVENTORY.md` for the full covered-vs-deferred matrix.

use vize_atelier_jsx::{JsxLang, VaporCompileOptions, compile_to_vapor};
use vize_carton::Bump;

/// Compile JSX to Vapor render code, asserting a single error-free component.
fn vapor(src: &str) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_vapor(&bump, src, JsxLang::Jsx, VaporCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

// ---------------------------------------------------------------------------
// Category: elements / intrinsic vs component resolution / fragments
// ---------------------------------------------------------------------------

#[test]
fn intrinsic_element_is_baked_into_a_template() {
    let code = vapor("const A = () => <div/>;");
    assert!(code.contains("_template(\"<div>"), "{code}");
    assert!(code.contains("export function render"), "{code}");
}

#[test]
fn component_resolves_and_uses_create_component_with_fallback() {
    let code = vapor("const A = () => <Comp/>;");
    assert!(code.contains("_resolveComponent(\"Comp\")"), "{code}");
    assert!(
        code.contains("_createComponentWithFallback(_component_Comp"),
        "{code}"
    );
}

#[test]
fn fragment_returns_an_array_of_template_nodes() {
    let code = vapor("const A = () => <><a/><b/></>;");
    assert!(code.contains("_template(\"<a>"), "{code}");
    assert!(code.contains("_template(\"<b>"), "{code}");
    assert!(code.contains("return [n0, n1]"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: attributes (static / dynamic / spread / class / style)
// ---------------------------------------------------------------------------

#[test]
fn static_attributes_are_baked_into_the_template() {
    let code = vapor("const A = () => <div class=\"a\" id=\"b\" data-x=\"y\"/>;");
    assert!(
        code.contains("<div class=\\\"a\\\" id=\\\"b\\\" data-x=\\\"y\\\">"),
        "{code}"
    );
}

#[test]
fn dynamic_bind_uses_render_effect_set_prop() {
    let code = vapor("const A = () => <div id={x}/>;");
    assert!(code.contains("_renderEffect("), "{code}");
    assert!(code.contains("_setProp(n0, \"id\", x)"), "{code}");
    assert!(!code.contains("_ctx.x"), "{code}");
}

#[test]
fn spread_uses_set_dynamic_props() {
    let code = vapor("const A = () => <div {...attrs}/>;");
    assert!(code.contains("_setDynamicProps(n0, [attrs])"), "{code}");
    assert!(!code.contains("_ctx.attrs"), "{code}");
}

#[test]
fn dynamic_class_uses_set_class() {
    let code = vapor("const A = () => <div class={c}/>;");
    assert!(code.contains("_setClass(n0, c)"), "{code}");
}

#[test]
fn dynamic_style_uses_set_style() {
    let code = vapor("const A = () => <div style={s}/>;");
    assert!(code.contains("_setStyle(n0, s)"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: children (text / interpolation / mixed)
// ---------------------------------------------------------------------------

#[test]
fn static_text_is_baked_into_the_template() {
    let code = vapor("const A = () => <div>hello</div>;");
    assert!(code.contains("_template(\"<div>hello</div>\""), "{code}");
}

#[test]
fn interpolation_uses_render_effect_and_set_text() {
    let code = vapor("const A = () => <div>{count}</div>;");
    assert!(code.contains("_renderEffect("), "{code}");
    assert!(code.contains("_setText("), "{code}");
    assert!(code.contains("_toDisplayString(count)"), "{code}");
    assert!(!code.contains("_ctx.count"), "{code}");
}

#[test]
fn mixed_text_and_interpolation_concatenates_in_set_text() {
    let code = vapor("const A = () => <div>Hi {name}!</div>;");
    assert!(
        code.contains("\"Hi \" + _toDisplayString(name) + \"!\""),
        "{code}"
    );
    assert!(code.contains("_setText("), "{code}");
}

#[test]
fn member_expression_interpolation_stays_bare() {
    let code = vapor("const A = () => <p>{user.name}</p>;");
    assert!(code.contains("_toDisplayString(user.name)"), "{code}");
    assert!(!code.contains("_ctx.user"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: control flow — conditional (&&, ternary), list (.map)
// ---------------------------------------------------------------------------

#[test]
fn logical_and_jsx_child_uses_create_if() {
    let code = vapor("const A = () => <ul>{ok && <span/>}</ul>;");
    assert!(code.contains("_createIf("), "{code}");
    assert!(code.contains("() => (ok)"), "{code}");
    assert!(code.contains("\"<span></span>\""), "{code}");
    assert!(!code.contains("_ctx."), "{code}");
}

#[test]
fn ternary_jsx_arms_use_create_if_with_two_branches() {
    let code = vapor("const A = () => <div>{ok ? <a/> : <b/>}</div>;");
    assert!(code.contains("_createIf("), "{code}");
    assert!(code.contains("\"<a></a>\""), "{code}");
    assert!(code.contains("\"<b></b>\""), "{code}");
}

#[test]
fn map_callback_uses_create_for() {
    let code = vapor("const A = () => <ul>{items.map((i) => <li/>)}</ul>;");
    assert!(code.contains("_createFor("), "{code}");
    assert!(code.contains("() => (items)"), "{code}");
    assert!(code.contains("\"<li></li>\""), "{code}");
    assert!(!code.contains("_ctx."), "{code}");
}

// ---------------------------------------------------------------------------
// Category: event handlers + option modifiers
// ---------------------------------------------------------------------------

#[test]
fn plain_event_handler_is_set_as_a_prop() {
    let code = vapor("const A = () => <button onClick={h}>go</button>;");
    assert!(code.contains("onClick"), "{code}");
    assert!(!code.contains("_ctx.h"), "{code}");
}

#[test]
fn capture_option_modifier_uses_on_with_capture_option() {
    // Vapor lowers `onClickCapture` to `_on(el, "click", invoker, { capture })`.
    let code = vapor("const A = () => <button onClickCapture={h}/>;");
    assert!(code.contains("_on(n0, \"click\""), "{code}");
    assert!(code.contains("capture: true"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: directives — v-model, v-show
// ---------------------------------------------------------------------------

#[test]
fn v_model_on_input_applies_text_model() {
    let code = vapor("const A = () => <input v-model={val}/>;");
    assert!(code.contains("_applyTextModel(n0"), "{code}");
}

#[test]
fn v_model_on_component_uses_model_value_prop() {
    let code = vapor("const A = () => <Input v-model={val}/>;");
    assert!(code.contains("modelValue: () => (val)"), "{code}");
    assert!(code.contains("\"onUpdate:modelValue\""), "{code}");
}

#[test]
fn v_show_applies_runtime_directive() {
    let code = vapor("const A = () => <div v-show={ok}>x</div>;");
    assert!(code.contains("_applyVShow(n0"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: object slots / scoped slots / default render-prop slot
// ---------------------------------------------------------------------------

#[test]
fn object_child_lowers_to_named_slot_function() {
    let code = vapor("const A = () => <Comp>{{ header: () => <h1>Hi</h1> }}</Comp>;");
    assert!(
        code.contains("_createComponentWithFallback(_component_Comp"),
        "{code}"
    );
    assert!(code.contains("\"header\": () =>"), "{code}");
    assert!(code.contains("_template(\"<h1>Hi</h1>\")"), "{code}");
}

// ---------------------------------------------------------------------------
// Category: module-level — preamble, multiple components, names
// ---------------------------------------------------------------------------

#[test]
fn generated_code_imports_template_from_vue() {
    let code = vapor("const A = () => <div>{count}</div>;");
    assert!(code.contains("from 'vue'"), "{code}");
    assert!(code.contains("_template"), "{code}");
}

#[test]
fn templates_are_exposed_on_the_component() {
    let bump = Bump::new();
    let out = compile_to_vapor(
        &bump,
        "const A = () => <div class=\"x\"/>;",
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
    assert!(!out.components[0].templates.is_empty());
    assert!(out.components[0].templates[0].contains("<div"));
}

#[test]
fn multiple_components_compile_independently_with_names() {
    let bump = Bump::new();
    let out = compile_to_vapor(
        &bump,
        "const A = () => <a/>;\nconst B = () => <b/>;",
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
    assert_eq!(out.components.len(), 2);
    assert_eq!(out.components[0].component_name.as_deref(), Some("A"));
    assert_eq!(out.components[1].component_name.as_deref(), Some("B"));
}
