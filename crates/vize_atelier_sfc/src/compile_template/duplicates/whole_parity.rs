//! Original full public Results against the untouched callback + legacy entry.
use crate::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    compile_sfc_for_adapter_with_experimental_options, compile_sfc_for_adapter_with_stage_capture,
    parse_sfc,
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};

const LATE_REFUSAL_SFC: &str = r#"<template><Widget><template #actions="v-slot:actions"><button>Run</button></template></Widget></template>"#;

#[test]
fn complete_original_dom_inline_module_maps_and_ssr_vapor_controls_match_retained() {
    let sources = [
        ("<template><div class='one'>雪🌸</div></template>", false),
        ("<template><div V-IF='a' v-if='b'></div></template>", false),
        (
            "<template><div :key..prop='a' :key.prop.='b'></div></template>",
            false,
        ),
        (
            "<template><div v-x:[a]foo='x' v-x:foo='y'></div></template>",
            false,
        ),
        (
            "<template><div v-if='a' v-if='b'><p CLASS='x' class='y'></p></div></template>",
            false,
        ),
        (
            "<template><div :one='a' :two='b' @click='a' @click='b'></div></template>",
            false,
        ),
        (
            "<template><div v-bind='a' v-bind='b' :one='a' v-bind:one='b'></div></template>",
            false,
        ),
        (
            "<template><div :[key].prop='a' :[key].prop='b'></div></template>",
            false,
        ),
        (
            "<template><div :key.a.b='a' :key.b.a='b' :key.a.b='c'></div></template>",
            false,
        ),
        (
            "<template><div v-pre :key='a' :key='b'><i class='x' CLASS='y'></i></div></template>",
            false,
        ),
        ("<template><div v-pre>{{</div></template>", false),
        (
            "<template><div @[event].stop='handler'></div></template>",
            false,
        ),
        (
            "<template><div title='<i id=x id=x>'><!-- <b id=x id=x> --></div></template>",
            false,
        ),
        (
            "<template><div id='a' id='b'><p v-if[a]='x' v-if[a]='y'/></div></template>",
            false,
        ),
        ("<template><div a= ></div></template>", false),
        (
            "<script setup>import Child from './Child.vue'; const value=1;</script><template><Child :value='value' /></template>",
            false,
        ),
        (
            "<script>export default {name:'Original'}</script><template><div id='a' id='b'></div></template>",
            false,
        ),
        (
            "<template><div class='item'></div></template><style scoped>.item{color:red}</style>",
            false,
        ),
        (
            "<template><title><g id='a' id='b'></g></title></template>",
            false,
        ),
        (
            "<template><svg><title><g id='a' id='b'></g></title></svg></template>",
            false,
        ),
        (
            "<template><svg><TITLE><g id='a' id='b'></g></TITLE></svg></template>",
            false,
        ),
        (
            "<template><svg><foreignObject><title><g id='a' id='b'></g></title></foreignObject></svg></template>",
            false,
        ),
        (
            "<template><div id='a' id='b'><svg><title><g id='x' id='y'></g></title></svg></div></template>",
            false,
        ),
        (
            "<template><div // v-x:[a]foo='x' v-x:foo='y'\n></div></template>",
            false,
        ),
        (
            "<template><div // v-x:[a]foo='x' v-x:foo='y'\n></div></template>",
            true,
        ),
        (LATE_REFUSAL_SFC, false),
    ];
    let mut pairs = 0;
    let mut populated_maps = 0;
    for (source, comments) in sources {
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: "Original雪🌸.vue".into(),
                ..Default::default()
            },
        )
        .unwrap();
        for shape in ["dom-inline", "dom-module", "ssr", "vapor"] {
            for source_map in [false, true] {
                let mut options = SfcCompileOptions::default();
                options.parse.filename = "Original雪🌸.vue".into();
                options.script.id = Some("Original雪🌸.vue".into());
                options.script.inline_template = shape == "dom-inline";
                options.template.ssr = shape == "ssr";
                options.vapor = shape == "vapor";
                options.template.compiler_options = Some(vize_atelier_dom::DomCompilerOptions {
                    experimental_in_tag_comments: comments,
                    ..Default::default()
                });
                let codegen = CodegenOptions {
                    source_map,
                    ..Default::default()
                };
                let output = if shape == "dom-inline" {
                    SfcScriptOutputMode::InlineTemplate
                } else {
                    SfcScriptOutputMode::SeparateTemplate
                };
                let compile = || {
                    compile_sfc_for_adapter_with_experimental_options(
                        &descriptor,
                        options.clone(),
                        TemplateSyntaxMode::Standard,
                        CustomElementMatcher::default(),
                        codegen.clone(),
                        output,
                        SfcCompileExperimentalOptions::default(),
                    )
                };
                let retained = vize_atelier_dom::differential::with_legacy_lane(compile);
                if retained
                    .as_ref()
                    .ok()
                    .is_some_and(|result| result.map.is_some())
                {
                    populated_maps += 1;
                }
                let expected = serde_json::to_value(&retained).unwrap();
                assert_eq!(
                    serde_json::to_value(compile()).unwrap(),
                    expected,
                    "{source} {shape} map={source_map}"
                );
                let captured = compile_sfc_for_adapter_with_stage_capture(
                    &descriptor,
                    options,
                    TemplateSyntaxMode::Standard,
                    CustomElementMatcher::default(),
                    codegen,
                    output,
                    SfcCompileExperimentalOptions::default(),
                )
                .map(|(result, _)| result);
                assert_eq!(
                    serde_json::to_value(captured).unwrap(),
                    expected,
                    "{source} {shape} map={source_map} captured"
                );
                pairs += 2;
            }
        }
    }
    assert_eq!(pairs, 416);
    assert!(populated_maps > 0);
}

#[test]
fn actual_post_parse_refusal_preserves_complete_results_and_checks_only_once() {
    use super::Preflight;
    use crate::SfcTemplateBlock;
    use vize_atelier_core::{CodegenExperimentalOptions, options::CodegenMode};
    use vize_atelier_dom::{
        DomCompilerOptions,
        differential::{take_emit_refusal, with_legacy_lane},
    };
    use vize_l0::dump::capture::{CaptureOutcome, StageCapture};
    use vize_l1_to_l2::{EmitError, UnsupportedReason};
    let source = LATE_REFUSAL_SFC
        .strip_prefix("<template>")
        .unwrap()
        .strip_suffix("</template>")
        .unwrap();
    let descriptor = parse_sfc(LATE_REFUSAL_SFC, SfcParseOptions::default()).unwrap();
    for captured in [false, true] {
        let arena = vize_l0::Allocator::default();
        let template = SfcTemplateBlock {
            content: std::borrow::Cow::Borrowed(source),
            loc: Default::default(),
            lang: None,
            src: None,
            attrs: Default::default(),
        };
        let options = DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            ..Default::default()
        };
        let (old_errors, old) = with_legacy_lane(|| {
            vize_atelier_dom::compile_sfc_template_with_custom_elements_template_syntax_hoisted_scope_id_sections_codegen_and_experimental_options_and_slot_scope(
            &arena, source, options.clone(), TemplateSyntaxMode::Standard, None, CustomElementMatcher::default(), CodegenOptions::default(), CodegenExperimentalOptions::default(), false,
        )
        });
        let _ = take_emit_refusal();
        let mut preflight = Preflight::new(&template, true, false);
        let mut capture = StageCapture::new("dom");
        let (errors, actual) = vize_atelier_dom::compile_sfc_template_with_preflight(
            &arena,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
            CodegenExperimentalOptions::default(),
            false,
            &mut preflight,
            captured.then_some(&mut capture),
        );
        assert_eq!((preflight.native_calls, preflight.retained_calls), (1, 0));
        assert!(preflight.finish().is_ok());
        assert_eq!(
            errors
                .iter()
                .map(|e| (e.code, e.message.as_str(), e.loc.as_ref()))
                .collect::<Vec<_>>(),
            old_errors
                .iter()
                .map(|e| (e.code, e.message.as_str(), e.loc.as_ref()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            (
                actual.result.code,
                actual.result.preamble,
                actual.result.map,
                actual.sections
            ),
            (
                old.result.code,
                old.result.preamble,
                old.result.map,
                old.sections
            )
        );
        if captured {
            assert_eq!(
                capture.outcome,
                CaptureOutcome::Legacy("emit-refused".into())
            );
            assert!(capture.pages.is_empty());
            // Captured compiles use the sidecar, not the ordinary differential note.
            assert_eq!(take_emit_refusal(), None);
        } else {
            assert_eq!(
                take_emit_refusal(),
                Some(EmitError::unsupported(
                    UnsupportedReason::PrefixExpressionRejected
                ))
            );
        }
        let compile = || {
            compile_sfc_for_adapter_with_experimental_options(
                &descriptor,
                SfcCompileOptions::default(),
                TemplateSyntaxMode::Standard,
                CustomElementMatcher::default(),
                CodegenOptions::default(),
                SfcScriptOutputMode::SeparateTemplate,
                SfcCompileExperimentalOptions::default(),
            )
        };
        let expected = serde_json::to_value(with_legacy_lane(compile)).unwrap();
        let actual = if captured {
            compile_sfc_for_adapter_with_stage_capture(
                &descriptor,
                SfcCompileOptions::default(),
                TemplateSyntaxMode::Standard,
                CustomElementMatcher::default(),
                CodegenOptions::default(),
                SfcScriptOutputMode::SeparateTemplate,
                SfcCompileExperimentalOptions::default(),
            )
            .map(|(result, _)| result)
        } else {
            compile()
        };
        assert_eq!(serde_json::to_value(actual).unwrap(), expected);
    }
}
