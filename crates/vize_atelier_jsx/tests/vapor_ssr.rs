//! JSX/TSX -> Vue Vapor SSR render codegen (#1533).

mod common;

use common::{snapshot_cases, snapshot_lang_cases, vapor_code};
use vize_atelier_jsx::{JsxLang, JsxOutputMode, VaporCompileOptions, compile_to_vapor};
use vize_carton::Bump;

#[test]
fn ssr_codegen_matrix() {
    let cases = [
        ("ssr render function", "const A = () => <div/>;"),
        ("server helpers", "const A = () => <div>{msg}</div>;"),
        ("static element", "const A = () => <p>hi</p>;"),
        (
            "static attribute",
            "const A = () => <div class=\"box\">x</div>;",
        ),
        ("dynamic attribute", "const A = () => <div id={x}/>;"),
        ("text interpolation", "const A = () => <div>{msg}</div>;"),
        (
            "static dynamic interpolation",
            "const A = () => <div id={x} class=\"box\">{msg}</div>;",
        ),
        ("member expression", "const A = () => <p>{user.name}</p>;"),
        (
            "logical and",
            "const A = () => <div>{cond && <span>yes</span>}</div>;",
        ),
        (
            "ternary",
            "const A = () => <div>{cond ? <span>a</span> : <em>b</em>}</div>;",
        ),
        (
            "map",
            "const A = () => <ul>{items.map((i) => <li>{i}</li>)}</ul>;",
        ),
        (
            "map components",
            "const A = () => <ul>{rows.map((row) => <Item data={row}/>)}</ul>;",
        ),
        (
            "nested ternary",
            "const A = () => <div>{a ? <p>A</p> : b ? <em>B</em> : <span>C</span>}</div>;",
        ),
        (
            "default scoped slot",
            "const A = () => <List>{(item) => <li>{item}</li>}</List>;",
        ),
        (
            "named object slots",
            "const A = () => <Comp>{{ header: () => <h1>H</h1>, default: () => <p>P</p> }}</Comp>;",
        ),
        (
            "nested component",
            "const A = () => <div><Child msg={x}/></div>;",
        ),
        (
            "control flow in slot",
            "const A = () => <List>{(rows) => rows.map((r) => <li>{r}</li>)}</List>;",
        ),
    ];

    insta::assert_snapshot!(snapshot_cases(&cases, |source| {
        vapor_code(source, JsxLang::Jsx, true)
    }));
}

#[test]
fn ssr_tsx_codegen_snapshot() {
    let cases = [(
        "tsx ssr",
        JsxLang::Tsx,
        "const A = (): JSX.Element => <span>{label}</span>;",
    )];

    insta::assert_snapshot!(snapshot_lang_cases(&cases, |source, lang| {
        vapor_code(source, lang, true)
    }));
}

#[test]
fn client_codegen_matrix_when_ssr_is_off() {
    let cases = [
        (
            "static dynamic interpolation",
            "const A = () => <div id={x} class=\"box\">{msg}</div>;",
        ),
        (
            "ternary",
            "const A = () => <div>{cond ? <span>a</span> : <em>b</em>}</div>;",
        ),
        (
            "map",
            "const A = () => <ul>{items.map((i) => <li>{i}</li>)}</ul>;",
        ),
        (
            "nested ternary",
            "const A = () => <div>{a ? <p>A</p> : b ? <em>B</em> : <span>C</span>}</div>;",
        ),
    ];

    insta::assert_snapshot!(snapshot_cases(&cases, |source| {
        vapor_code(source, JsxLang::Jsx, false)
    }));
}

#[test]
fn ssr_component_metadata_snapshot() {
    let bump = Bump::new();
    let out = compile_to_vapor(
        &bump,
        "const Widget = () => <div/>;",
        JsxLang::Jsx,
        VaporCompileOptions { ssr: true },
    );

    assert_eq!(out.components[0].component_name.as_deref(), Some("Widget"));
    assert_eq!(out.components[0].mode, JsxOutputMode::Vapor);
    assert!(out.components[0].templates.is_empty());
}

#[test]
fn client_and_ssr_outputs_differ_for_the_same_source() {
    let src = "const A = () => <div>{msg}</div>;";

    assert_ne!(
        vapor_code(src, JsxLang::Jsx, true).as_str(),
        vapor_code(src, JsxLang::Jsx, false).as_str()
    );
}
