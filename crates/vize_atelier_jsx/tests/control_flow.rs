//! JSX control-flow expression children -> real v-if / v-for VNodes.
#![expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]

#[expect(dead_code, reason = "shared test helpers; each binary uses a subset")]
mod common;

use common::{snapshot_cases, vapor_code, vdom_code};
use vize_atelier_jsx::JsxLang;

#[test]
fn vdom_control_flow_matrix() {
    let cases = [
        (
            "logical and with jsx",
            "const A = () => <ul>{ok && <li/>}</ul>;",
        ),
        (
            "conditional jsx arms",
            "const A = () => <div>{ok ? <a/> : <b/>}</div>;",
        ),
        (
            "map callback",
            "const A = () => <ul>{items.map((i) => <li>{i}</li>)}</ul>;",
        ),
        (
            "map callback index alias",
            "const A = () => <ul>{rows.map((row, idx) => <li key={idx}>{row}</li>)}</ul>;",
        ),
        ("plain expression", "const A = () => <div>{count}</div>;"),
        (
            "non jsx logical and",
            "const A = () => <div>{a && b}</div>;",
        ),
        (
            "nested ternary alternate",
            "const A = () => <div>{a ? <p/> : b ? <em/> : <span/>}</div>;",
        ),
        (
            "logical and arm inside ternary",
            "const A = () => <div>{a ? <p/> : (cond && <span/>)}</div>;",
        ),
    ];

    insta::assert_snapshot!(
        "vdom_control_flow_matrix_vdom_child_values_6890",
        snapshot_cases(&cases, |source| { vdom_code(source, JsxLang::Jsx) })
    );
}

#[test]
fn vapor_control_flow_matrix() {
    let cases = [
        (
            "logical and with jsx",
            "const A = () => <ul>{ok && <span/>}</ul>;",
        ),
        (
            "map callback",
            "const A = () => <ul>{items.map((i) => <li/>)}</ul>;",
        ),
        (
            "nested ternary alternate",
            "const A = () => <div>{a ? <p/> : b ? <em/> : <span/>}</div>;",
        ),
    ];

    insta::assert_snapshot!(snapshot_cases(&cases, |source| {
        vapor_code(source, JsxLang::Jsx, false)
    }));
}

/// `renderList` runs only the returned JSX per item and passes `(item, index)`,
/// so any other callback is rejected instead of silently dropping its
/// statements or parameters (#12477).
#[test]
fn map_callbacks_beyond_a_list_body_are_rejected() {
    const MESSAGE: &str = "a `.map()` callback that renders JSX must be an expression body or a single `return`, with at most `(item, index)` parameters; move other statements out of the callback";
    for source in [
        "const A = () => <ul>{items.map((item) => { const label = item.name; return <li>{label}</li>; })}</ul>;",
        "const A = () => <ul>{items.map((item, i, all) => <li>{all.length}</li>)}</ul>;",
        "const A = () => <ul>{items.map((...args) => <li>{args[0]}</li>)}</ul>;",
        "const A = () => <ul>{items.map(function (item) { if (!item) return null; return <li>{item}</li>; })}</ul>;",
    ] {
        let allocator = vize_l0::Allocator::new();
        let out =
            vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);
        let errors: Vec<&str> = out
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.is_error())
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert_eq!(errors, [MESSAGE], "{source}");
    }
    let list = "const A = () => <ul>{items.map((item) => { return <li>{item}</li>; })}</ul>;";
    assert_eq!(
        vdom_code(list, JsxLang::Jsx),
        r#"export function render(_ctx, _cache) {
  return (_createElementVNode("ul", null, [
    (_createElementVNode(_Fragment, null, _renderList(items, (item) => {
      return (_createElementVNode("li", null, [
        item
      ]))
    }), 256 /* UNKEYED_FRAGMENT */))
  ]))
}"#
    );
}
