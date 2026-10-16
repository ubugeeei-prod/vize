//! Complete product results, including code/import preambles and populated maps.

use super::corpus;
use crate::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    compile_sfc_for_adapter_with_experimental_options, parse_sfc,
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};

#[test]
fn native_descriptor_projection_preserves_all_four_product_profiles_and_maps() {
    let pack = corpus();
    let mut populated_maps = 0;
    let mut compared = 0;
    for case in pack["cases"].as_array().expect("cases") {
        let source = case["source"].as_str().expect("original source");
        let parse = SfcParseOptions {
            filename: "Native雪🌸.vue".into(),
            ..Default::default()
        };
        let actual = parse_sfc(source, parse.clone());
        let retained = vize_croquis::sfc::parse_sfc(source, parse.clone());
        let (Ok(actual), Ok(retained)) = (&actual, &retained) else {
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::to_value(retained).unwrap()
            );
            continue;
        };
        for shape in ["dom-inline", "dom-module", "vapor", "ssr"] {
            for source_map in [false, true] {
                let mut options = SfcCompileOptions::default();
                options.parse = parse.clone();
                options.script.id = Some("Native雪🌸.vue".into());
                options.script.inline_template = shape == "dom-inline";
                options.template.ssr = shape == "ssr";
                options.vapor = shape == "vapor";
                let compile = |descriptor| {
                    compile_sfc_for_adapter_with_experimental_options(
                        descriptor,
                        options.clone(),
                        TemplateSyntaxMode::Standard,
                        CustomElementMatcher::default(),
                        CodegenOptions {
                            source_map,
                            ..Default::default()
                        },
                        if shape == "dom-inline" {
                            SfcScriptOutputMode::InlineTemplate
                        } else {
                            SfcScriptOutputMode::SeparateTemplate
                        },
                        SfcCompileExperimentalOptions::default(),
                    )
                };
                let expected = compile(retained);
                let output = compile(actual);
                if expected.as_ref().ok().is_some_and(|r| r.map.is_some()) {
                    populated_maps += 1;
                }
                assert_eq!(
                    serde_json::to_value(output).unwrap(),
                    serde_json::to_value(expected).unwrap(),
                    "{} {shape} map={source_map}",
                    case["id"]
                );
                compared += 1;
            }
        }
    }
    assert!(
        compared >= 160,
        "complete original profile comparisons: {compared}"
    );
    assert!(
        populated_maps > 0,
        "the map contract must include populated maps"
    );
}
