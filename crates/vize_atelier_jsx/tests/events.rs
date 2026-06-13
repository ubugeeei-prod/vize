//! JSX event handler option-modifiers (feature C).
//!
//! babel-plugin-jsx encodes event option modifiers as a name suffix:
//! `onClickCapture` / `onClickOnce` / `onClickPassive` (composable). These are
//! lowered to a `v-on` directive so the core codegen emits the suffixed
//! listener key, while plain `onClick` (no recognized suffix) stays a `v-bind`
//! exactly as before.

mod common;

use common::{as_directive, lower_one, root_element, simple_content, vdom_code};
use vize_atelier_jsx::JsxLang;
use vize_carton::Bump;

#[test]
fn event_modifier_codegen_snapshot() {
    let cases = [
        ("capture", "const A = () => <button onClickCapture={h}/>;"),
        ("once", "const A = () => <button onClickOnce={h}/>;"),
        (
            "passive capture",
            "const A = () => <input onInputPassiveCapture={h}/>;",
        ),
        ("plain", "const A = () => <button onClick={h}/>;"),
    ];

    insta::assert_snapshot!(common::snapshot_cases(&cases, |source| {
        vdom_code(source, JsxLang::Jsx)
    }));
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
