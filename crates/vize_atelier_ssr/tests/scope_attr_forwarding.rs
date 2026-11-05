//! #11921: scope belongs to the runtime VNode, never a forwarded $attrs bag.
#![cfg(feature = "legacy-differential")]

use serde_json::{Value, json};
use vize_atelier_core::CodegenOptions;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    TemplateCompileOptions, compile_sfc_for_adapter, parse_sfc,
};
use vize_atelier_ssr::differential::{record_lanes, with_legacy_lane};
use vize_l0::cstr;

macro_rules! source {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/compiler/ssr-component-scope-attrs-11921/",
            $name,
            ".vue.txt"
        ))
    };
}

const FILES: &[(&str, &str)] = &[
    ("Leaf", source!("Leaf")),
    ("Frame", source!("Frame")),
    ("Panel", source!("Panel")),
    ("App", source!("App")),
];

fn compile(name: &str, source: &str, ssr: bool) -> Value {
    let filename = cstr!("{name}.vue");
    let parse = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("parse complete reported SFC");
    let scope_id = match name {
        "App" => Some("app".into()),
        "Panel" => Some("panel".into()),
        _ => None,
    };
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            parse,
            script: ScriptCompileOptions {
                id: Some(filename.clone()),
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename),
                ssr,
                is_prod: true,
                ..Default::default()
            },
            scope_id,
            ..Default::default()
        },
        Default::default(),
        Default::default(),
        CodegenOptions {
            source_map: true,
            prefix_identifiers: true,
            ..Default::default()
        },
        SfcScriptOutputMode::SeparateTemplate,
    )
    .expect("compile complete reported SFC");
    assert!(result.errors.is_empty(), "{name}: {:?}", result.errors);
    assert!(result.warnings.is_empty(), "{name}: {:?}", result.warnings);
    assert!(
        result.map.is_some(),
        "retain complete source map for {name}"
    );
    serde_json::to_value(result).expect("retain every public compiler field")
}

#[test]
fn source_scoped_component_graph_keeps_attrs_separate_on_both_ssr_routes() {
    let mut cases = Vec::new();
    for case in [
        "reported",
        "no-forwarded-attrs",
        "dynamic-root",
        "inherited-attrs",
    ] {
        let mut modules = Vec::new();
        for &(name, original) in FILES {
            let source = match (case, name) {
                ("no-forwarded-attrs", "Panel") => original.replace(" v-bind=\"$attrs\"", ""),
                ("dynamic-root", "App") => original.replace("<Panel ", "<component :is=\"Panel\" "),
                ("inherited-attrs", "Panel") => {
                    original.replace("defineOptions({ inheritAttrs: false });", "")
                }
                _ => original.to_owned(),
            };
            let (selected, lanes) = record_lanes(|| compile(name, &source, true));
            assert_eq!(
                lanes,
                ["s4"],
                "{case}/{name}: selected SSR plan owns source"
            );
            let legacy = with_legacy_lane(|| compile(name, &source, true));
            assert_eq!(
                selected, legacy,
                "{case}/{name}: all fields and source maps"
            );
            let client = compile(name, &source, false);
            modules.push(json!({
                "name": name,
                "source": source,
                "scopeId": match name {
                    "App" => Some("data-v-app"),
                    "Panel" => Some("data-v-panel"),
                    _ => None,
                },
                "selected": selected,
                "legacy": legacy,
                "client": client,
            }));
        }
        cases.push(json!({"name": case, "modules": modules}));
    }
    if let Some(destination) = std::env::var_os("VIZE_SCOPE_ATTR_CAPTURE") {
        std::fs::write(
            destination,
            serde_json::to_vec_pretty(&cases).expect("serialize complete source graphs"),
        )
        .expect("retain source graphs for real Vue SSR, hydration and CSR");
    }
}
