//! Vapor-backend JSX/TSX parity suite (Part of #1491).

mod common;

use common::{snapshot_cases, vapor_code};
use vize_atelier_jsx::{JsxLang, VaporCompileOptions, compile_to_vapor};
use vize_carton::Bump;

#[test]
fn vapor_parity_matrix_snapshot() {
    let cases = [
        ("intrinsic element", "const A = () => <div/>;"),
        ("component", "const A = () => <Comp/>;"),
        ("fragment", "const A = () => <><a/><b/></>;"),
        (
            "static attributes",
            "const A = () => <div class=\"a\" id=\"b\" data-x=\"y\"/>;",
        ),
        ("dynamic bind", "const A = () => <div id={x}/>;"),
        ("spread", "const A = () => <div {...attrs}/>;"),
        ("dynamic class", "const A = () => <div class={c}/>;"),
        ("dynamic style", "const A = () => <div style={s}/>;"),
        ("static text", "const A = () => <div>hello</div>;"),
        ("interpolation", "const A = () => <div>{count}</div>;"),
        ("mixed text", "const A = () => <div>Hi {name}!</div>;"),
        ("member expression", "const A = () => <p>{user.name}</p>;"),
        (
            "logical and child",
            "const A = () => <ul>{ok && <span/>}</ul>;",
        ),
        (
            "ternary arms",
            "const A = () => <div>{ok ? <a/> : <b/>}</div>;",
        ),
        (
            "map callback",
            "const A = () => <ul>{items.map((i) => <li/>)}</ul>;",
        ),
        (
            "plain event",
            "const A = () => <button onClick={h}>go</button>;",
        ),
        (
            "capture event",
            "const A = () => <button onClickCapture={h}/>;",
        ),
        ("v-model input", "const A = () => <input v-model={val}/>;"),
        (
            "v-model component",
            "const A = () => <Input v-model={val}/>;",
        ),
        ("v-show", "const A = () => <div v-show={ok}>x</div>;"),
        (
            "object child slot",
            "const A = () => <Comp>{{ header: () => <h1>Hi</h1> }}</Comp>;",
        ),
    ];

    insta::assert_snapshot!(snapshot_cases(&cases, |source| {
        vapor_code(source, JsxLang::Jsx, false)
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
fn multiple_components_compile_independently_with_names() {
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
