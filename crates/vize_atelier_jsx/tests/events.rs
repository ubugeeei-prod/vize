//! JSX event handler option-modifiers (feature C).
//!
//! babel-plugin-jsx encodes event option modifiers as a name suffix:
//! `onClickCapture` / `onClickOnce` / `onClickPassive` (composable). These are
//! lowered to a `v-on` directive so the core codegen emits the suffixed
//! listener key, while plain `onClick` (no recognized suffix) stays a `v-bind`
//! exactly as before.

mod common;

use common::{as_directive, lower_one, root_element, simple_content};
use vize_atelier_jsx::{DomCompileOptions, JsxLang, compile_to_dom};
use vize_carton::Bump;

fn dom(src: &str) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_dom(&bump, src, JsxLang::Jsx, DomCompileOptions::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.components.len(), 1, "expected one component");
    out.components.into_iter().next().unwrap().code
}

#[test]
fn capture_modifier_yields_capture_listener_key() {
    let code = dom("const A = () => <button onClickCapture={h}/>;");
    assert!(code.contains("onClickCapture: h"), "{code}");
}

#[test]
fn once_modifier_yields_once_listener_key() {
    let code = dom("const A = () => <button onClickOnce={h}/>;");
    assert!(code.contains("onClickOnce: h"), "{code}");
}

#[test]
fn composed_passive_capture_yields_combined_listener_key() {
    let code = dom("const A = () => <input onInputPassiveCapture={h}/>;");
    assert!(code.contains("onInputPassiveCapture: h"), "{code}");
}

#[test]
fn plain_on_click_stays_a_bare_bind_prop() {
    // Regression: no recognized suffix -> plain `v-bind:onClick`, unchanged.
    let code = dom("const A = () => <button onClick={h}/>;");
    assert!(code.contains("onClick: h"), "{code}");
}

#[test]
fn capture_modifier_lowers_to_a_v_on_directive() {
    let bump = Bump::new();
    let root = lower_one(&bump, "const a = <button onClickCapture={h}/>;");
    let element = root_element(&root);
    assert_eq!(element.props.len(), 1, "expected one prop");

    let directive = as_directive(&element.props[0]);
    assert_eq!(directive.name, "on", "directive name");

    let arg = directive.arg.as_ref().expect("v-on arg");
    assert_eq!(simple_content(arg), "click", "event name");

    assert_eq!(directive.modifiers.len(), 1, "one modifier");
    assert_eq!(
        directive.modifiers[0].content.as_str(),
        "capture",
        "modifier content"
    );
}
