//! JSX control-flow expression children -> real v-if / v-for VNodes.

mod common;

use common::{dom_code, snapshot_cases, vapor_code};
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

    insta::assert_snapshot!(snapshot_cases(&cases, |source| {
        dom_code(source, JsxLang::Jsx)
    }));
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
