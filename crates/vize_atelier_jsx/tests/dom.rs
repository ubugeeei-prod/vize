//! JSX/TSX -> Vue VDOM compilation (#1493).
//!
//! These pin the generated render modules as snapshots. Exact snapshots are
//! intentionally used instead of substring assertions so helper imports, patch
//! flags, prop shapes, and closure semantics all move together in review.

mod common;

use common::{dom_code, snapshot_lang_cases};
use vize_atelier_jsx::{DomCompileOptions, JsxLang, JsxOutputMode, compile_to_dom};
use vize_carton::Bump;

#[test]
fn vdom_codegen_matrix() {
    let cases = [
        ("single element", JsxLang::Jsx, "const A = () => <div/>;"),
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
        ("component", JsxLang::Jsx, "const A = () => <Comp/>;"),
        (
            "event handler",
            JsxLang::Jsx,
            "const A = () => <button onClick={h}/>;",
        ),
        ("fragment", JsxLang::Jsx, "const A = () => <><a/><b/></>;"),
        (
            "v-if",
            JsxLang::Jsx,
            "const A = () => <div v-if={ok}>x</div>;",
        ),
        (
            "v-for",
            JsxLang::Jsx,
            "const A = () => <ul><li v-for={(i) in items}>{i}</li></ul>;",
        ),
        (
            "v-model",
            JsxLang::Jsx,
            "const A = () => <input v-model={val}/>;",
        ),
        (
            "dynamic style",
            JsxLang::Jsx,
            "const A = () => <div style={s}/>;",
        ),
        (
            "spread props",
            JsxLang::Jsx,
            "const A = () => <div {...attrs}/>;",
        ),
        (
            "tsx",
            JsxLang::Tsx,
            "const A = (): JSX.Element => <p>{msg}</p>;",
        ),
    ];

    insta::assert_snapshot!(snapshot_lang_cases(&cases, dom_code));
}

#[test]
fn multiple_components_each_compile_with_their_name() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = () => <a/>;\nconst B = () => <b/>;",
        JsxLang::Jsx,
        DomCompileOptions::default(),
    );

    assert_eq!(out.components.len(), 2);
    assert_eq!(out.components[0].component_name.as_deref(), Some("A"));
    assert_eq!(out.components[1].component_name.as_deref(), Some("B"));
    insta::assert_snapshot!(format!(
        "## component 0 preamble\n{}## component 0 code\n{}\n\n## component 1 preamble\n{}## component 1 code\n{}",
        out.components[0].preamble,
        out.components[0].code,
        out.components[1].preamble,
        out.components[1].code
    ));
}

#[test]
fn mode_defaults_to_vdom() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = () => <div/>;",
        JsxLang::Jsx,
        DomCompileOptions::default(),
    );

    assert_eq!(out.components[0].mode, JsxOutputMode::Vdom);
}

#[test]
fn vapor_directive_is_recorded_on_the_component() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = () => { \"use vue:vapor\"; return <div/>; };",
        JsxLang::Jsx,
        DomCompileOptions::default(),
    );

    assert_eq!(out.components[0].mode, JsxOutputMode::Vapor);
}

#[test]
fn preamble_imports_runtime_helpers() {
    let bump = Bump::new();
    let out = compile_to_dom(
        &bump,
        "const A = () => <div>{x}</div>;",
        JsxLang::Jsx,
        DomCompileOptions::default(),
    );

    insta::assert_snapshot!(out.components[0].preamble.as_str());
}
