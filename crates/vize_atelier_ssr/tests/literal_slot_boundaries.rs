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
fn public_dom_vapor_ssr_keep_literal_and_outlet_slot_ownership() {
    let mut files = Vec::new();
    for &(name, source, literal) in CASES {
        for kind in ["dom", "vapor", "ssr"] {
            let (current, lanes) = record_lanes(|| compile(name, source, kind, "selected"));
            if kind == "ssr" {
                let legacy = with_legacy_lane(|| compile(name, source, kind, "explicit-legacy"));
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
            let code = current.code.as_str();
            let outlet_helper = match kind {
                "dom" => "renderSlot",
                "vapor" => "createSlot",
                "ssr" => "ssrRenderSlot",
                _ => unreachable!(),
            };
            assert_eq!(
                code.contains(outlet_helper),
                !literal,
                "{name}/{kind}: {code}"
            );
            if literal {
                let emitted_literal = match kind {
                    "dom" => {
                        code.contains("_createElementBlock(\"slot\"")
                            || code.contains("_createElementVNode(\"slot\"")
                            || code.contains("_createStaticVNode(\"<slot")
                    }
                    "vapor" => {
                        code.contains("_template(\"<slot")
                            || code.contains("_template(\"<div><slot")
                    }
                    "ssr" => code.contains("<slot") && code.contains("</slot>"),
                    _ => unreachable!(),
                };
                assert!(
                    emitted_literal,
                    "{name}/{kind}: actual literal emitter: {code}"
                );
                assert!(!code.contains("v-pre"), "{name}/{kind}: marker is consumed");
            }
        }
    }
    super::v_pre_boundaries::observe_runtime(
        serde_json::json!({ "files": files }),
        "../../tests/tooling/support/ssr-literal-slot-boundaries.mjs",
        "ssr-literal-slot-boundaries",
        "SSR_LITERAL_SLOT_RUNTIME_PACKET",
        8,
    );
}
