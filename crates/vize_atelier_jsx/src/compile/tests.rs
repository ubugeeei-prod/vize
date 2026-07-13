#![allow(clippy::disallowed_macros)]

use super::*;

#[test]
fn parses_named_import_specifiers_and_source() {
    assert_eq!(
        parse_named_import("import { a as _a, b as _b } from \"vue\""),
        Some((" a as _a, b as _b ", "vue"))
    );
    assert_eq!(
        parse_named_import("import { x } from 'vue'"),
        Some((" x ", "vue"))
    );
    assert_eq!(parse_named_import("const _hoisted = 1"), None);
    assert_eq!(parse_named_import("import Foo from \"bar\""), None);
}

#[test]
fn merge_preambles_dedups_overlapping_vue_imports() {
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
    let bump = Bump::new();
    let out = compile_jsx(
        &bump,
        "const A = () => <div>{x}</div>;",
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_snapshot!(out.module_code());
    });
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
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_snapshot!(single.source_map().expect("single component carries a map"));
    });
    let module = single.module_code();
    let map = single
        .module_source_map(module.as_str(), "App.jsx")
        .expect("assembled module carries a relocated map");
    let document: serde_json::Value = serde_json::from_str(map.as_str()).unwrap();
    assert_eq!(document["file"], "App.jsx");
    assert_eq!(document["sources"], serde_json::json!(["App.jsx"]));
    assert_eq!(
        document["sourcesContent"],
        serde_json::json!(["const A = () => <div>{x}</div>;"])
    );
    assert!(document["mappings"].as_str().unwrap().starts_with(';'));
    let multi = compile_jsx(
        &bump,
        "const A = () => <div>{x}</div>;\nconst B = () => <span>{y}</span>;",
        JsxLang::Jsx,
        &config,
    );
    assert!(multi.components.len() >= 2);
    assert!(multi.source_map().is_none());
}
