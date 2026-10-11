//! Separate public compiler controls; original runtime populations are unchanged.

use super::{
    CodegenOptions, CustomElementMatcher, ScriptCompileOptions, SfcCompileExperimentalOptions,
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, StyleCompileOptions,
    TemplateCompileOptions, TemplateSyntaxMode, compile_sfc_for_adapter_with_experimental_options,
    parse_sfc, record_lanes, with_legacy_lane,
};

const CASES: &[(&str, &str, bool)] = &[
    (
        "own-empty",
        "<template><slot v-pre></slot></template>",
        true,
    ),
    (
        "inherited-empty",
        "<template><div v-pre><slot></slot></div></template>",
        true,
    ),
    ("normal-outlet", "<template><slot></slot></template>", false),
    (
        "original-nonempty",
        "<template><slot v-pre>{{ not }} an interpolation</slot></template>",
        true,
    ),
];

fn compile(
    name: &str,
    source: &str,
    kind: &str,
    route: &str,
) -> vize_atelier_sfc::SfcCompileResult {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("whole source");
    let options = SfcCompileOptions {
        parse: SfcParseOptions::default(),
        script: ScriptCompileOptions::default(),
        template: TemplateCompileOptions {
            ssr: kind == "ssr",
            ..Default::default()
        },
        style: StyleCompileOptions::default(),
        vapor: kind == "vapor",
        scope_id: None,
    };
    let codegen = CodegenOptions {
        source_map: true,
        prefix_identifiers: true,
        ..Default::default()
    };
    let experimental = SfcCompileExperimentalOptions::default();
    println!(
        "SSR_LITERAL_SLOT_OPTIONS name={name:?} kind={kind:?} options={options:?} codegen={codegen:?} experimental={experimental:?}"
    );
    let result = compile_sfc_for_adapter_with_experimental_options(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        codegen,
        SfcScriptOutputMode::SeparateTemplate,
        experimental,
    );
    println!(
        "SSR_LITERAL_SLOT_PUBLIC_PACKET name={name:?} source={source:?} kind={kind:?} route={route:?} result={result:?}"
    );
    if let Ok(result) = &result {
        println!(
            "SSR_LITERAL_SLOT_PUBLIC_JSON={}",
            serde_json::json!({ "name": name, "source": source, "kind": kind, "route": route, "result": result })
        );
    }
    result.expect("complete public compiler result")
}

#[test]
fn public_ssr_keeps_compiler_owned_literal_and_outlet_slot_ownership() {
    let expected: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/compiler/v-pre-literal-boundary/public-slot.expected.json"
    )))
    .expect("sixteen complete fixed public compiler packets");
    assert_eq!(expected.as_array().expect("whole module pins").len(), 16);
    let expected = expected
        .as_array()
        .unwrap()
        .iter()
        .filter(|packet| packet["kind"] == "ssr")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(expected.len(), 8, "the same eight historical SSR packets");
    let mut observed = Vec::new();
    let mut files = Vec::new();
    for &(name, source, _literal) in CASES {
        for kind in ["ssr"] {
            let (current, lanes) = record_lanes(|| compile(name, source, kind, "selected"));
            observed.push(serde_json::json!({
                "name": name, "source": source, "kind": kind,
                "route": "selected", "result": current
            }));
            if kind == "ssr" {
                let legacy = with_legacy_lane(|| compile(name, source, kind, "explicit-legacy"));
                observed.push(serde_json::json!({
                    "name": name, "source": source, "kind": kind,
                    "route": "explicit-legacy", "result": legacy
                }));
                assert_eq!(
                    serde_json::to_value(&current).expect("whole current"),
                    serde_json::to_value(&legacy).expect("whole legacy")
                );
                assert_eq!(
                    lanes,
                    if name == "original-nonempty" {
                        ["legacy.element"]
                    } else {
                        ["s4"]
                    }
                );
                files.push(serde_json::json!({
                    "name": name, "source": source, "current": current, "legacy": legacy
                }));
            }
            assert!(
                current.errors.is_empty(),
                "{name}/{kind}: {:?}",
                current.errors
            );
            assert!(
                current.warnings.is_empty(),
                "{name}/{kind}: {:?}",
                current.warnings
            );
        }
    }
    assert_eq!(
        serde_json::json!(observed),
        serde_json::json!(expected),
        "every complete module/map field"
    );
    super::v_pre_boundaries::observe_runtime(
        serde_json::json!({ "files": files }),
        "../../tests/tooling/support/ssr-literal-slot-boundaries.mjs",
        "ssr-literal-slot-boundaries",
        "SSR_LITERAL_SLOT_RUNTIME_PACKET",
        8,
    );
}

#[test]
fn public_dom_and_vapor_preserve_actual_ca_default_modules() {
    // This baseline is copied from the authentic old provider observation,
    // never from corrective current output. The historical sixteen-row file
    // stays immutable; its six post-regression literal non-SSR rows are history.
    let expected: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/compiler/v-pre-literal-boundary/public-default-ca.expected.json"
    ))).expect("eight authentic-ca default public packets");
    assert_eq!(expected.as_array().unwrap().len(), 8);
    let mut observed = Vec::new();
    for &(name, source, _) in CASES {
        for kind in ["dom", "vapor"] {
            let result = compile(name, source, kind, "original-default");
            observed
                .push(serde_json::json!({"name":name,"source":source,"kind":kind,"result":result}));
        }
    }
    assert_eq!(
        serde_json::json!(observed),
        expected,
        "whole old/default modules, maps, errors and warnings"
    );
}
