//! JSX/TSX -> Vue VDOM compilation (#1493).
//!
//! These assert the structure of the generated render code (helper calls, patch
//! flags, prop shapes) rather than byte-for-byte `@vue/babel-plugin-jsx`
//! parity: Vize emits through its own codegen/runtime-helper path, so hoisting
//! and block-tree details are intentionally Vize-shaped.

use vize_atelier_jsx::{DomCompileOptions, JsxLang, JsxOutputMode, compile_to_dom};
use vize_carton::Bump;

fn dom(src: &str) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_dom(&bump, src, JsxLang::Jsx, DomCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

#[test]
fn single_element_uses_create_element_block() {
    let code = dom("const A = () => <div/>;");
    assert!(code.contains("_createElementBlock(\"div\")"), "{code}");
    assert!(code.contains("_openBlock()"), "{code}");
}

#[test]
fn static_attribute_is_emitted_as_prop() {
    let code = dom("const A = () => <div class=\"a\"/>;");
    assert!(code.contains("{ class: \"a\" }"), "{code}");
}

#[test]
fn interpolation_uses_to_display_string_with_text_flag() {
    let code = dom("const A = () => <div>{count}</div>;");
    assert!(code.contains("_toDisplayString(count)"), "{code}");
    assert!(code.contains("1 /* TEXT */"), "{code}");
}

#[test]
fn identifiers_are_not_ctx_prefixed() {
    // JSX render fns close over setup scope; expressions stay bare.
    let code = dom("const A = () => <div>{count}</div>;");
    assert!(!code.contains("_ctx.count"), "{code}");
}

#[test]
fn component_is_resolved_and_blocked() {
    let code = dom("const A = () => <Comp/>;");
    assert!(code.contains("_resolveComponent(\"Comp\")"), "{code}");
    assert!(code.contains("_createBlock(_component_Comp"), "{code}");
}

#[test]
fn event_handler_stays_an_on_prop() {
    let code = dom("const A = () => <button onClick={h}/>;");
    assert!(code.contains("onClick: h"), "{code}");
}

#[test]
fn fragment_uses_fragment_helper() {
    let code = dom("const A = () => <><a/><b/></>;");
    assert!(code.contains("_Fragment"), "{code}");
    assert!(code.contains("64 /* STABLE_FRAGMENT */"), "{code}");
}

#[test]
fn v_if_compiles_to_a_conditional() {
    let code = dom("const A = () => <div v-if={ok}>x</div>;");
    assert!(code.contains("(ok)"), "{code}");
    assert!(code.contains("_createCommentVNode"), "{code}");
}

#[test]
fn v_for_compiles_to_render_list() {
    let code = dom("const A = () => <ul><li v-for={(i) in items}>{i}</li></ul>;");
    assert!(code.contains("_renderList(items"), "{code}");
    assert!(code.contains("_Fragment"), "{code}");
}

#[test]
fn v_model_expands_to_update_handler_and_directive() {
    let code = dom("const A = () => <input v-model={val}/>;");
    assert!(code.contains("\"onUpdate:modelValue\""), "{code}");
    assert!(code.contains("_vModelText"), "{code}");
    assert!(code.contains("_withDirectives"), "{code}");
}

#[test]
fn dynamic_style_is_normalized() {
    let code = dom("const A = () => <div style={s}/>;");
    assert!(code.contains("_normalizeStyle(s)"), "{code}");
    assert!(code.contains("4 /* STYLE */"), "{code}");
}

#[test]
fn spread_props_merge() {
    let code = dom("const A = () => <div {...attrs}/>;");
    assert!(code.contains("attrs"), "{code}");
}

#[test]
fn emits_an_exported_render_function() {
    let code = dom("const A = () => <div/>;");
    assert!(code.contains("export function render"), "{code}");
}

#[test]
fn multiple_components_each_compile_with_their_name() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = () => <a/>;\nconst B = () => <b/>;",
        JsxLang::Jsx,
        DomCompileOptions::default(),
    );
    assert_eq!(out.components.len(), 2);
    assert_eq!(out.components[0].component_name.as_deref(), Some("A"));
    assert_eq!(out.components[1].component_name.as_deref(), Some("B"));
}

#[test]
fn mode_defaults_to_vdom() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = () => <div/>;",
        JsxLang::Jsx,
        DomCompileOptions::default(),
    );
    assert_eq!(out.components[0].mode, JsxOutputMode::Vdom);
}

#[test]
fn vapor_directive_is_recorded_on_the_component() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = () => { \"use vue:vapor\"; return <div/>; };",
        JsxLang::Jsx,
        DomCompileOptions::default(),
    );
    // The mode override is surfaced even though this entry point emits VDOM;
    // the Vapor backend (#1494) consumes it.
    assert_eq!(out.components[0].mode, JsxOutputMode::Vapor);
}

#[test]
fn tsx_module_compiles_to_vdom() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = (): JSX.Element => <p>{msg}</p>;",
        JsxLang::Tsx,
        DomCompileOptions::default(),
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    assert!(out.components[0].code.contains("_toDisplayString(msg)"));
}

#[test]
fn preamble_imports_runtime_helpers() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = () => <div>{x}</div>;",
        JsxLang::Jsx,
        DomCompileOptions::default(),
    );
    let preamble = &out.components[0].preamble;
    assert!(preamble.contains("from \"vue\""), "{preamble}");
    assert!(preamble.contains("toDisplayString"), "{preamble}");
}
