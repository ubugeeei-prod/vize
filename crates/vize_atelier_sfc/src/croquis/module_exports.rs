//! Demand-only module export admission without another script parse.

use super::analysis::{
    DescriptorAnalysisMode, DescriptorObservation, analyze_sfc_descriptor_resolved_impl,
};
use super::{SfcCroquisAnalysis, SfcCroquisOptions};
use crate::SfcDescriptor;
use vize_atelier_core::RootNode;
use vize_croquis::script_parser::AuthoredModuleExport;

/// Analyze the same original blocks and return separate empty-export metadata.
/// Existing public option/result models and their serialized forms are unchanged.
#[doc(hidden)]
pub fn analyze_sfc_descriptor_with_empty_module_exports(
    descriptor: &SfcDescriptor<'_>,
    root: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> (SfcCroquisAnalysis, bool) {
    let (analysis, exports) = analyze_sfc_descriptor_with_module_exports(
        descriptor,
        root,
        options,
        options_api,
        legacy_vue2,
    );
    (analysis, exports.is_some_and(|exports| exports.is_empty()))
}

/// Analyze the same original blocks and return every authored named export of
/// the SFC module, or `None` when its export set is not provable from the
/// original scripts alone. A star re-export of the component can then name
/// exactly these exports instead of also exposing generated declarations.
#[doc(hidden)]
pub fn analyze_sfc_descriptor_with_module_exports(
    descriptor: &SfcDescriptor<'_>,
    root: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> (SfcCroquisAnalysis, Option<Vec<AuthoredModuleExport>>) {
    let mut exports = None;
    let analysis = analyze_sfc_descriptor_resolved_impl::<false>(
        descriptor,
        root,
        options,
        DescriptorAnalysisMode {
            options_api,
            legacy_vue2,
            include_script_content: true,
        },
        None,
        None,
        DescriptorObservation {
            module_exports: Some(&mut exports),
            ..Default::default()
        },
    );
    if (!options.merge_scripts && descriptor.script.is_some() && descriptor.script_setup.is_some())
        || !descriptor_module_sources_are_known(descriptor)
    {
        exports = None;
    }
    (analysis, exports)
}

/// Preserve admission before a caller prepares or drops source blocks.
#[doc(hidden)]
pub fn descriptor_module_sources_are_known(descriptor: &SfcDescriptor<'_>) -> bool {
    [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
        .into_iter()
        .flatten()
        .all(|block| {
            block.src.is_none()
                && block.lang.as_deref().is_none_or(|lang| {
                    matches!(lang.trim(), "js" | "ts" | "jsx" | "tsx" | "mjs" | "mts")
                })
        })
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::disallowed_macros,
        reason = "fixtures assert complete debug and source contracts"
    )]

    use super::{
        analyze_sfc_descriptor_with_empty_module_exports, descriptor_module_sources_are_known,
    };
    use crate::croquis::{
        SfcCroquisAnalysis, SfcCroquisOptions, analyze_sfc_descriptor_with_context,
        analyze_sfc_descriptor_with_context_legacy_vue2,
        analyze_sfc_descriptor_with_context_options_api,
    };
    use crate::{SfcParseOptions, parse_sfc};
    use vize_l0::Allocator;

    fn assert_capture(
        source: &str,
        options: SfcCroquisOptions,
        mode: (bool, bool),
        expected_empty: bool,
    ) -> SfcCroquisAnalysis {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let allocator = Allocator::default();
        let root = descriptor.template.as_ref().map(|template| {
            let (root, errors) = vize_armature::parse(&allocator, &template.content);
            assert!(errors.is_empty());
            root
        });
        let ordinary = if mode.1 {
            analyze_sfc_descriptor_with_context_legacy_vue2(&descriptor, root.as_ref(), options)
        } else if mode.0 {
            analyze_sfc_descriptor_with_context_options_api(&descriptor, root.as_ref(), options)
        } else {
            analyze_sfc_descriptor_with_context(&descriptor, root.as_ref(), options)
        };
        let (captured, empty) = analyze_sfc_descriptor_with_empty_module_exports(
            &descriptor,
            root.as_ref(),
            options,
            mode.0,
            mode.1,
        );
        assert_eq!(empty, expected_empty, "source={source}, mode={mode:?}");
        assert_eq!(format!("{ordinary:#?}"), format!("{captured:#?}"));
        assert_eq!(ordinary.croquis.to_vir(), captured.croquis.to_vir());
        assert_eq!(
            serde_json::to_value(ordinary.croquis.semantic_snapshot()).unwrap(),
            serde_json::to_value(captured.croquis.semantic_snapshot()).unwrap()
        );
        assert_eq!(ordinary.script_content, captured.script_content);
        assert_eq!(ordinary.script_offset, captured.script_offset);
        captured
    }

    #[test]
    fn genuine_no_script_descriptor_is_known_empty_without_inventing_a_script_observation() {
        let source = "<template><p>雪</p></template><style scoped>p { color: red; }</style><i18n lang=\"json\">{\"label\":\"Name\"}</i18n>";
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        assert!(descriptor.script.is_none() && descriptor.script_setup.is_none());
        assert!(descriptor_module_sources_are_known(&descriptor));
        let analysis = assert_capture(source, SfcCroquisOptions::full(), (false, false), true);
        assert_eq!(analysis.script_content, None);
    }

    #[test]
    fn authentic_normal_and_setup_blocks_use_the_complete_authored_star_export_set() {
        for (script, normal_empty, setup_empty) in [
            (
                "interface Private { label: string }; const local = '雪';",
                true,
                true,
            ),
            (
                "const component = {}; export { component as default };",
                true,
                false,
            ),
            ("export {};", true, false),
            ("export default { name: 'OnlyDefault' };", true, false),
            ("export default function Component() {}", true, false),
            ("export default class Component {}", true, false),
            ("export type Public = string;", false, false),
            ("export interface Public { label: string }", false, false),
            ("const local = 1; export { local };", false, false),
            ("export const local = 1;", false, false),
            ("export { default } from './Other.vue';", false, false),
            (
                "import Other from './Other.vue'; export { Other as default };",
                false,
                false,
            ),
            ("export * from './Other.vue';", false, false),
            ("export type * from './types';", false, false),
            ("export * as Namespace from './Other.vue';", false, false),
            ("const broken = ;", false, false),
            ("const slots = {}; slots.;", false, false),
            (
                "namespace Unsupported { export const value = 1; }",
                false,
                false,
            ),
        ] {
            for (setup, expected) in [("", normal_empty), (" setup", setup_empty)] {
                let source = format!(
                    "<script{setup} lang=\"ts\">{script}</script><template><p>Content</p></template>"
                );
                assert_capture(&source, SfcCroquisOptions::full(), (false, false), expected);
            }
        }
    }

    #[test]
    fn both_actual_blocks_must_be_empty_and_either_block_can_stickily_refuse() {
        let empty = "interface Private { label: string }; const hidden = 1;";
        for (plain, setup, expected) in [
            ("export default {};", empty, true),
            (
                "const component = {}; export { component as default };",
                empty,
                true,
            ),
            (empty, "export {};", false),
            (empty, "export default {};", false),
            (
                empty,
                "const component = {}; export { component as default };",
                false,
            ),
            ("export interface Public { label: string }", empty, false),
            (empty, "export interface Public { label: string }", false),
            ("export const publicValue = 1;", empty, false),
            (empty, "export const publicValue = 1;", false),
            ("export * from './Other.vue';", empty, false),
            (empty, "export type * from './types';", false),
            ("const broken = ;", empty, false),
            (empty, "const slots = {}; slots.;", false),
        ] {
            for setup_first in [false, true] {
                let plain = format!("<script lang=\"ts\">{plain}</script>");
                let setup = format!("<script setup lang=\"ts\">{setup}</script>");
                let source = if setup_first {
                    format!("{setup}{plain}")
                } else {
                    format!("{plain}{setup}")
                };
                assert_capture(&source, SfcCroquisOptions::full(), (false, false), expected);
            }
        }
    }

    #[test]
    fn disabled_analysis_and_unobserved_sibling_blocks_refuse_but_single_blocks_stay_valid() {
        let plain = "<script lang=\"ts\">export default {};</script>";
        let setup = "<script setup lang=\"ts\">const local = 1;</script>";
        let both = format!("{plain}{setup}");
        let unmerged = SfcCroquisOptions::full().without_script_merge();
        assert_capture(&both, unmerged, (false, false), false);
        for source in [plain, setup] {
            assert_capture(source, unmerged, (false, false), true);
        }
        let mut disabled = SfcCroquisOptions::full();
        disabled.analyzer_options.analyze_script = false;
        for source in [plain, setup, &both, "<template><p>no script</p></template>"] {
            assert_capture(source, disabled, (false, false), false);
        }
    }

    #[test]
    fn original_foreign_and_external_blocks_refuse_even_when_their_inline_text_is_empty() {
        for source in [
            "<script lang=\"coffee\">const local = 1;</script>",
            "<script setup lang=\"rust\">const local = 1;</script>",
            "<script src=\"./External.ts\"></script>",
            "<script setup src=\"./External.ts\"></script>",
            "<script src=\"./External.ts\">export default {};</script><script setup>const local = 1;</script>",
            "<script>export default {};</script><script setup lang=\"coffee\">const local = 1;</script>",
        ] {
            let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
            assert!(!descriptor_module_sources_are_known(&descriptor));
            assert_capture(source, SfcCroquisOptions::full(), (false, false), false);
        }
    }

    #[test]
    fn accepted_js_ts_jsx_tsx_dialects_use_the_original_script_parse() {
        for lang in ["js", "ts", "mjs", "mts", "jsx", "tsx"] {
            for setup in ["", " setup"] {
                let source =
                    format!("<script{setup} lang=\"{lang}\">const label = 'value';</script>");
                let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
                assert!(descriptor_module_sources_are_known(&descriptor));
                assert_capture(&source, SfcCroquisOptions::full(), (false, false), true);
            }
        }
        let jsx = "<script setup lang=\"tsx\">const node = <section>Content</section>;</script>";
        assert_capture(jsx, SfcCroquisOptions::full(), (false, false), true);
    }

    #[test]
    fn capture_preserves_complete_options_api_modes_macros_template_and_merged_offsets() {
        let source = "<script lang=\"ts\">import PlainCard from './PlainCard.vue'; export default { inheritAttrs: false, data() { return { title: 'plain' }; } };</script>\r\n<script setup lang=\"ts\">import { ref } from 'vue'; interface Private { label: string }; const props = defineProps<Private>(); const count = ref(1);</script>\r\n<template><PlainCard :label=\"props.label\" /><p>{{ count }}</p></template>";
        for mode in [(false, false), (true, false), (false, true)] {
            for options in [
                SfcCroquisOptions::full(),
                SfcCroquisOptions::for_compile(),
                SfcCroquisOptions::for_declaration(),
                SfcCroquisOptions::lint_demand().with_unused_bindings(),
            ] {
                let captured = assert_capture(source, options, mode, true);
                assert!(captured.croquis.bindings.contains("PlainCard"));
                assert!(captured.croquis.bindings.contains("count"));
                let script = captured.script_content.as_deref().unwrap();
                let (start, end) = captured.croquis.binding_spans["count"];
                assert_eq!(script.get(start as usize..end as usize), Some("count"));
                let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
                assert_eq!(
                    captured.script_source_offset(&descriptor, start),
                    source.find("count =").unwrap() as u32
                );
            }
        }
    }
}
