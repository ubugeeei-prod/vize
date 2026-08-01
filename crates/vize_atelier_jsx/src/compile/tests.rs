//! Tests for [`super::compile`].
//!
//! Split out of that module so it stays inside the per-file source-length
//! budget.

use super::*;

#[test]
fn module_code_prepends_merged_preamble_to_render_code() {
    // A single VDOM component's module string is its preamble followed by the
    // render code, so the emitted helpers are actually imported.
    let bump = Bump::new();
    let out = compile_jsx(
        &bump,
        "const A = () => <div>{x}</div>;",
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();
    insta::assert_snapshot!(module);
}

#[test]
fn source_map_present_only_for_single_component_module() {
    let bump = Bump::new();
    let mut config = JsxCompileConfig::default();
    config.vdom.source_map = true;

    let single = compile_jsx(
        &bump,
        "const A = () => <div>{x}</div>;",
        JsxLang::Jsx,
        &config,
    );
    assert_eq!(single.components.len(), 1);
    let map = single.source_map().expect("single component carries a map");
    insta::assert_snapshot!(map);

    let multi = compile_jsx(
        &bump,
        "const A = () => <div>{x}</div>;\nconst B = () => <span>{y}</span>;",
        JsxLang::Jsx,
        &config,
    );
    assert!(multi.components.len() >= 2);
    assert!(
        multi.source_map().is_none(),
        "multi-component module reports no map to avoid misalignment"
    );
}
