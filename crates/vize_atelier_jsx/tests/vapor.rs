//! JSX/TSX -> Vue Vapor compilation (#1494).
//!
//! Asserts the structure of the generated Vapor code (templates, render
//! effects, helper calls) and, crucially, that JSX free identifiers stay bare
//! (closure semantics) rather than being `_ctx.`-prefixed.

use vize_atelier_jsx::{JsxLang, JsxOutputMode, VaporCompileOptions, compile_to_vapor};
use vize_carton::Bump;

fn vapor(src: &str) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_vapor(&bump, src, JsxLang::Jsx, VaporCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

#[test]
fn element_produces_a_template() {
    let code = vapor("const A = () => <div/>;");
    assert!(code.contains("_template(\"<div>"), "{code}");
    assert!(code.contains("export function render"), "{code}");
}

#[test]
fn static_attribute_is_baked_into_the_template() {
    let code = vapor("const A = () => <div class=\"a\"/>;");
    assert!(code.contains("<div class=\\\"a\\\">"), "{code}");
}

#[test]
fn interpolation_uses_render_effect_and_set_text() {
    let code = vapor("const A = () => <div>{count}</div>;");
    assert!(code.contains("_renderEffect("), "{code}");
    assert!(code.contains("_setText("), "{code}");
    assert!(code.contains("_toDisplayString(count)"), "{code}");
}

#[test]
fn free_identifiers_stay_bare_not_ctx_prefixed() {
    // The render runs inside the component closure: no `_ctx.`.
    let code = vapor("const A = () => <div>{count}</div>;");
    assert!(!code.contains("_ctx.count"), "{code}");
}

#[test]
fn member_expression_stays_bare() {
    let code = vapor("const A = () => <p>{user.name}</p>;");
    assert!(code.contains("_toDisplayString(user.name)"), "{code}");
    assert!(!code.contains("_ctx.user"), "{code}");
}

#[test]
fn event_handler_is_set_as_a_bare_prop() {
    let code = vapor("const A = () => <button onClick={h}>go</button>;");
    assert!(code.contains("onClick"), "{code}");
    assert!(!code.contains("_ctx.h"), "{code}");
}

#[test]
fn dynamic_binding_uses_set_prop() {
    let code = vapor("const A = () => <div id={x}/>;");
    assert!(code.contains("_renderEffect("), "{code}");
    assert!(!code.contains("_ctx.x"), "{code}");
}

#[test]
fn generated_code_imports_from_vue() {
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
fn mode_defaults_to_vapor() {
    let bump = Bump::new();
    let out = compile_to_vapor(
        &bump,
        "const A = () => <div/>;",
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
    assert_eq!(out.components[0].mode, JsxOutputMode::Vapor);
}

#[test]
fn use_vue_vapor_directive_is_recorded() {
    let bump = Bump::new();
    let out = compile_to_vapor(
        &bump,
        "const A = () => { \"use vue:vapor\"; return <div/>; };",
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
    assert_eq!(out.components[0].mode, JsxOutputMode::Vapor);
}

#[test]
fn component_name_is_resolved() {
    let bump = Bump::new();
    let out = compile_to_vapor(
        &bump,
        "const Widget = () => <div/>;",
        JsxLang::Jsx,
        VaporCompileOptions::default(),
    );
    assert_eq!(out.components[0].component_name.as_deref(), Some("Widget"));
}

#[test]
fn tsx_compiles_to_vapor() {
    let bump = Bump::new();
    let out = compile_to_vapor(
        &bump,
        "const A = (): JSX.Element => <span>{label}</span>;",
        JsxLang::Tsx,
        VaporCompileOptions::default(),
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(out.components[0].code.contains("_toDisplayString(label)"));
}

#[test]
fn multiple_components_compile_independently() {
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
