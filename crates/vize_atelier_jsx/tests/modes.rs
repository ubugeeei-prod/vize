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
fn unrelated_prologue_produces_no_diagnostic() {
    // `"use strict"` is a legitimate directive, not a malformed Vize one.
    let bump = Bump::new();
    let out = lower_source(
        &bump,
        "const App = () => { \"use strict\"; return <div/>; };",
        JsxLang::Jsx,
    );
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
}

#[test]
fn malformed_vue_directive_is_diagnosed() {
    // A typo'd suffix (`vdomm`) opens with `use vue:` so it is almost certainly
    // a mistyped mode directive; report it instead of silently ignoring it.
    let bump = Bump::new();
    let src = "const App = () => { \"use vue:vdomm\"; return <div/>; };";
    let out = lower_source(&bump, src, JsxLang::Jsx);
    assert!(out.has_errors(), "expected a diagnostic for the typo");
    let diag = out
        .diagnostics
        .iter()
        .find(|d| d.message.as_str().contains("use vue:vdomm"))
        .expect("diagnostic should name the offending directive");
    assert!(diag.message.as_str().contains("use vue:vdom"));
    assert!(diag.message.as_str().contains("use vue:vapor"));
    // The range maps back into the original source.
    assert!(diag.end > diag.start);
    assert_eq!(
        &src[diag.start as usize..diag.end as usize],
        "\"use vue:vdomm\""
    );
    // The unknown directive does not select a mode.
    assert_eq!(out.roots.len(), 1);
    assert_eq!(out.roots[0].mode, None);
}

#[test]
fn conflicting_directives_are_diagnosed() {
    // Two different mode directives in one component cannot both apply.
    let bump = Bump::new();
    let src = "const App = () => { \"use vue:vapor\"; \"use vue:vdom\"; return <div/>; };";
    let out = lower_source(&bump, src, JsxLang::Jsx);
    assert!(out.has_errors(), "expected a conflict diagnostic");
    let diag = out
        .diagnostics
        .iter()
        .find(|d| {
            d.message
                .as_str()
                .contains("conflicting JSX mode directives")
        })
        .expect("a conflict diagnostic should be produced");
    // The diagnostic points at the second, conflicting directive.
    assert_eq!(
        &src[diag.start as usize..diag.end as usize],
        "\"use vue:vdom\""
    );
    // The first directive still wins for the component's resolved mode.
    assert_eq!(out.roots[0].mode, Some(JsxOutputMode::Vapor));
}

#[test]
fn repeated_identical_directives_do_not_conflict() {
    // Redundant but not contradictory: no diagnostic.
    let bump = Bump::new();
    let src = "const App = () => { \"use vue:vapor\"; \"use vue:vapor\"; return <div/>; };";
    let out = lower_source(&bump, src, JsxLang::Jsx);
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.roots[0].mode, Some(JsxOutputMode::Vapor));
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
