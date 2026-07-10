//! JSX/TSX -> Vue SSR compilation (#1580).

mod common;

use common::{snapshot_cases, snapshot_lang_cases};
use vize_atelier_jsx::{JsxLang, JsxOutputMode, SsrCompileOptions, compile_to_ssr};
use vize_carton::Bump;

fn ssr_code(source: &str, lang: JsxLang) -> vize_carton::String {
    let bump = Bump::new();
    let out = compile_to_ssr(&bump, source, lang, SsrCompileOptions::default());
    assert!(
        !out.has_errors(),
        "unexpected diagnostics: {:?}",
        out.diagnostics
    );
    assert_eq!(out.components.len(), 1, "expected exactly one component");
    out.components.into_iter().next().unwrap().code
}

#[test]
fn ssr_codegen_matrix() {
    let cases = [
        ("element", "const A = () => <div/>;"),
        ("text", "const A = () => <p>hello</p>;"),
        ("interpolation", "const A = () => <div>{msg}</div>;"),
        ("dynamic attr", "const A = () => <div id={id}>{msg}</div>;"),
        (
            "control flow",
            "const A = () => <div>{ok ? <span>yes</span> : <em>no</em>}</div>;",
        ),
        (
            "list",
            "const A = () => <ul>{items.map((item) => <li>{item}</li>)}</ul>;",
        ),
    ];

    insta::assert_snapshot!(snapshot_cases(&cases, |source| {
        ssr_code(source, JsxLang::Jsx)
    }));
}

#[test]
fn ssr_tsx_codegen_snapshot() {
    let cases = [(
        "tsx ssr",
        JsxLang::Tsx,
        "const A = (): JSX.Element => <span>{label}</span>;",
    )];

    insta::assert_snapshot!(snapshot_lang_cases(&cases, ssr_code));
}

#[test]
fn ssr_preserves_component_metadata_and_client_mode() {
    let bump = Bump::new();
    let out = compile_to_ssr(
        &bump,
        "const Fast = () => { \"use vue:vapor\"; return <div/>; };",
        JsxLang::Jsx,
        SsrCompileOptions::default(),
    );

    assert_eq!(out.components[0].component_name.as_deref(), Some("Fast"));
    assert_eq!(out.components[0].mode, JsxOutputMode::Vapor);
}
