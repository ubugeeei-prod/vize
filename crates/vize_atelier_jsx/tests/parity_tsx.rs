//! TSX-syntax + mode-directive parity suite (Part of #1491).
//!
//! Covers the TSX-specific surface (type annotations, generic component calls,
//! `as` casts, non-null assertions) plus the `"use vue:vapor"` / `"use vue:vdom"`
//! mode-directive prologue and mixed VDOM/Vapor modules — verifying TSX flows
//! cleanly through both backends and that per-component mode selection holds.

use vize_atelier_jsx::{
    DomCompileOptions, JsxLang, JsxOutputMode, VaporCompileOptions, compile_to_dom,
    compile_to_vapor, lower_source,
};
use vize_carton::Bump;

fn dom_tsx(src: &str) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_dom(&bump, src, JsxLang::Tsx, DomCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

fn vapor_tsx(src: &str) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_vapor(&bump, src, JsxLang::Tsx, VaporCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

// ---------------------------------------------------------------------------
// Category: TSX type syntax flows through to both backends
// ---------------------------------------------------------------------------

#[test]
fn typed_arrow_component_compiles_to_vdom() {
    let code = dom_tsx("const A = (props: { id: number }): JSX.Element => <div id={props.id}/>;");
    assert!(code.contains("id: props.id"), "{code}");
    assert!(code.contains("8 /* PROPS */"), "{code}");
}

#[test]
fn typed_arrow_component_compiles_to_vapor() {
    let code = vapor_tsx("const A = (): JSX.Element => <span>{label}</span>;");
    assert!(code.contains("_toDisplayString(label)"), "{code}");
    assert!(!code.contains("_ctx.label"), "{code}");
}

#[test]
fn generic_component_call_resolves_as_a_component() {
    // `<List<number> .../>` — the type argument is stripped, `List` resolves.
    let code = dom_tsx("const A = () => <List<number> items={xs}/>;");
    assert!(code.contains("_resolveComponent(\"List\")"), "{code}");
    assert!(code.contains("items: xs"), "{code}");
}

#[test]
fn as_cast_inside_interpolation_is_type_stripped_in_codegen() {
    // The `as` cast survives lowering (see tests/tsx.rs) but VDOM codegen runs
    // the expression through the JS code generator, which drops the type cast.
    let code = dom_tsx("const A = () => <p>{(x as string)}</p>;");
    assert!(code.contains("_toDisplayString(x)"), "{code}");
}

#[test]
fn non_null_assertion_in_binding_is_type_stripped_in_codegen() {
    // The `!` assertion is recovered at the IR level (tests/tsx.rs) but codegen
    // strips it; the binding still emits as a dynamic `PROPS` prop.
    let code = dom_tsx("const A = () => <div title={el!}/>;");
    assert!(code.contains("title: el"), "{code}");
    assert!(code.contains("8 /* PROPS */"), "{code}");
}

#[test]
fn tsx_type_annotation_is_an_error_when_parsed_as_plain_jsx() {
    // The same source is a hard parse error in `.jsx` mode — guards the lang flag.
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const A = (props: { id: number }) => <div/>;",
        JsxLang::Jsx,
    );
    assert!(out.has_errors(), "expected a JSX-mode parse error");
}

// ---------------------------------------------------------------------------
// Category: mode directives + mixed VDOM/Vapor module
// ---------------------------------------------------------------------------

#[test]
fn default_mode_is_vdom_for_dom_backend() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = () => <div/>;",
        JsxLang::Tsx,
        DomCompileOptions::default(),
    );
    assert_eq!(out.components[0].mode, JsxOutputMode::Vdom);
}

#[test]
fn use_vue_vapor_directive_is_surfaced_on_component_mode() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = () => { \"use vue:vapor\"; return <div/>; };",
        JsxLang::Tsx,
        DomCompileOptions::default(),
    );
    assert_eq!(out.components[0].mode, JsxOutputMode::Vapor);
}

#[test]
fn use_vue_vdom_directive_overrides_vapor_default() {
    let bump = Bump::new();
    let out = compile_to_vapor(
        &bump,
        "const A = () => { \"use vue:vdom\"; return <div/>; };",
        JsxLang::Tsx,
        VaporCompileOptions::default(),
    );
    assert_eq!(out.components[0].mode, JsxOutputMode::Vdom);
}

#[test]
fn mixed_module_selects_mode_per_component() {
    // One Vapor-directed component and one default component in the same module;
    // lowering records the per-component mode override.
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const A = () => { \"use vue:vapor\"; return <a/>; };\nconst B = () => <b/>;",
        JsxLang::Tsx,
    );
    assert_eq!(out.roots.len(), 2);
    assert_eq!(out.roots[0].mode, Some(JsxOutputMode::Vapor));
    assert_eq!(out.roots[0].component_name.as_deref(), Some("A"));
    assert_eq!(out.roots[1].mode, None);
    assert_eq!(out.roots[1].component_name.as_deref(), Some("B"));
}
