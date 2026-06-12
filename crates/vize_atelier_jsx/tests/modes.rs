//! Component-boundary detection: output-mode directives and component names.

mod common;

use common::lower_single;
use vize_atelier_jsx::{JsxLang, JsxOutputMode, lower_source};
use vize_carton::Bump;

fn jsx<'a>(bump: &'a Bump, src: &str) -> vize_atelier_jsx::LoweredRoot<'a> {
    lower_single(bump, src, JsxLang::Jsx)
}

#[test]
fn no_directive_means_default_mode() {
    let bump = Bump::new();
    let lowered = jsx(&bump, "const App = () => <div/>;");
    assert_eq!(lowered.mode, None);
}

#[test]
fn use_vue_vapor_prologue_selects_vapor() {
    let bump = Bump::new();
    let lowered = jsx(
        &bump,
        "const Fast = () => { \"use vue:vapor\"; return <div/>; };",
    );
    assert_eq!(lowered.mode, Some(JsxOutputMode::Vapor));
}

#[test]
fn use_vue_vdom_prologue_selects_vdom() {
    let bump = Bump::new();
    let lowered = jsx(
        &bump,
        "function Slow() { \"use vue:vdom\"; return <div/>; }",
    );
    assert_eq!(lowered.mode, Some(JsxOutputMode::Vdom));
}

#[test]
fn unrelated_prologue_is_ignored() {
    let bump = Bump::new();
    let lowered = jsx(
        &bump,
        "const App = () => { \"use strict\"; return <div/>; };",
    );
    assert_eq!(lowered.mode, None);
}

#[test]
fn arrow_component_name_is_resolved() {
    let bump = Bump::new();
    let lowered = jsx(&bump, "const MyButton = () => <button/>;");
    assert_eq!(lowered.component_name.as_deref(), Some("MyButton"));
}

#[test]
fn function_declaration_name_is_resolved() {
    let bump = Bump::new();
    let lowered = jsx(&bump, "function Card() { return <div/>; }");
    assert_eq!(lowered.component_name.as_deref(), Some("Card"));
}

#[test]
fn nested_function_uses_innermost_directive() {
    let bump = Bump::new();
    // Outer is vdom, inner arrow overrides to vapor for its own JSX.
    let src = "function Outer() {\n  \"use vue:vdom\";\n  const Inner = () => { \"use vue:vapor\"; return <span/>; };\n  return Inner;\n}";
    let lowered = jsx(&bump, src);
    assert_eq!(lowered.mode, Some(JsxOutputMode::Vapor));
    assert_eq!(lowered.component_name.as_deref(), Some("Inner"));
}

#[test]
fn directive_modes_apply_per_component_in_a_module() {
    let bump = Bump::new();
    let src = "const A = () => { \"use vue:vapor\"; return <a/>; };\nconst B = () => <b/>;";
    let out = lower_source(&bump, src, JsxLang::Jsx);
    assert_eq!(out.roots.len(), 2);
    assert_eq!(out.roots[0].mode, Some(JsxOutputMode::Vapor));
    assert_eq!(out.roots[0].component_name.as_deref(), Some("A"));
    assert_eq!(out.roots[1].mode, None);
    assert_eq!(out.roots[1].component_name.as_deref(), Some("B"));
}

#[test]
fn jsx_outside_any_function_has_no_component_name() {
    let bump = Bump::new();
    let lowered = jsx(&bump, "const node = <div/>;");
    assert_eq!(lowered.component_name, None);
    assert_eq!(lowered.mode, None);
}
