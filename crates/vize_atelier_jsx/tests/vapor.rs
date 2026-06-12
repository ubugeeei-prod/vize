//! JSX/TSX -> Vue Vapor compilation (#1494).

mod common;

use common::{snapshot_lang_cases, vapor_code};
use vize_atelier_jsx::{JsxLang, JsxOutputMode, VaporCompileOptions, compile_to_vapor};
use vize_carton::Bump;

#[test]
fn vapor_codegen_matrix() {
    let cases = [
        ("element", JsxLang::Jsx, "const A = () => <div/>;"),
        (
            "static attribute",
            JsxLang::Jsx,
            "const A = () => <div class=\"a\"/>;",
        ),
        (
            "interpolation",
            JsxLang::Jsx,
            "const A = () => <div>{count}</div>;",
        ),
        (
            "member expression",
            JsxLang::Jsx,
            "const A = () => <p>{user.name}</p>;",
        ),
        (
            "event handler",
            JsxLang::Jsx,
            "const A = () => <button onClick={h}>go</button>;",
        ),
        (
            "dynamic binding",
            JsxLang::Jsx,
            "const A = () => <div id={x}/>;",
        ),
        (
            "tsx",
            JsxLang::Tsx,
            "const A = (): JSX.Element => <span>{label}</span>;",
        ),
    ];

    insta::assert_snapshot!(snapshot_lang_cases(&cases, |source, lang| {
        vapor_code(source, lang, false)
    }));
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

    assert_eq!(out.components[0].templates, vec!["<div class=\"x\"></div>"]);
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
