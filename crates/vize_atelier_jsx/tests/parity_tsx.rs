//! TSX-syntax + mode-directive parity suite (Part of #1491).

mod common;

use common::{dom_code, snapshot_cases, vapor_code};
use vize_atelier_jsx::{
    DomCompileOptions, JsxLang, JsxOutputMode, VaporCompileOptions, compile_to_dom,
    compile_to_vapor, lower_source,
};
use vize_carton::Bump;

#[test]
fn tsx_vdom_codegen_matrix() {
    let cases = [
        (
            "typed arrow component",
            "const A = (props: { id: number }): JSX.Element => <div id={props.id}/>;",
        ),
        (
            "generic component call",
            "const A = () => <List<number> items={xs}/>;",
        ),
        (
            "as cast interpolation",
            "const A = () => <p>{(x as string)}</p>;",
        ),
        ("non null binding", "const A = () => <div title={el!}/>;"),
    ];

    insta::assert_snapshot!(snapshot_cases(&cases, |source| {
        dom_code(source, JsxLang::Tsx)
    }));
}

#[test]
fn tsx_vapor_codegen_matrix() {
    let cases = [(
        "typed arrow component",
        "const A = (): JSX.Element => <span>{label}</span>;",
    )];

    insta::assert_snapshot!(snapshot_cases(&cases, |source| {
        vapor_code(source, JsxLang::Tsx, false)
    }));
}

#[test]
fn tsx_type_annotation_is_an_error_when_parsed_as_plain_jsx() {
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const A = (props: { id: number }) => <div/>;",
        JsxLang::Jsx,
    );

    assert!(out.has_errors(), "expected a JSX-mode parse error");
    insta::assert_debug_snapshot!(out.diagnostics);
}

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
