//! Tests for [`super::compile`].
//!
//! Split out of that module so it stays inside the per-file source-length
//! budget.

use super::*;

#[test]
fn parses_named_import_specifiers_and_source() {
    assert_eq!(
        parse_named_import("import { a as _a, b as _b } from \"vue\""),
        Some((" a as _a, b as _b ", "vue"))
    );
    // Single-quoted source is accepted too.
    assert_eq!(
        parse_named_import("import { x } from 'vue'"),
        Some((" x ", "vue"))
    );
    // Non-imports and namespace/default imports are not brace-named imports.
    assert_eq!(parse_named_import("const _hoisted = 1"), None);
    assert_eq!(parse_named_import("import Foo from \"bar\""), None);
}

#[test]
fn merge_preambles_dedups_overlapping_vue_imports() {
    // Two components importing overlapping helpers from "vue" must collapse to
    // one import with each binding declared exactly once (concatenating the
    // raw lines would redeclare `_createElementBlock`, an ESM error).
    let merged = merge_preambles(
        [
            "import { createElementBlock as _createElementBlock } from \"vue\"\n",
            "import { createElementBlock as _createElementBlock, toDisplayString as _toDisplayString } from \"vue\"\n",
        ]
        .into_iter(),
    );
    assert_eq!(
        merged,
        "import { createElementBlock as _createElementBlock, toDisplayString as _toDisplayString } from \"vue\"\n"
    );
}

#[test]
fn merge_preambles_keeps_distinct_sources_and_hoists() {
    // Distinct sources each get their own import (first-seen order), and a
    // non-import hoist line is preserved verbatim after the imports.
    let merged = merge_preambles(
        [
            "import { a as _a } from \"vue\"\nconst _hoisted = 1\n",
            "import { b as _b } from \"other\"\n",
        ]
        .into_iter(),
    );
    assert_eq!(
        merged,
        "import { a as _a } from \"vue\"\nimport { b as _b } from \"other\"\nconst _hoisted = 1\n"
    );
}

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
