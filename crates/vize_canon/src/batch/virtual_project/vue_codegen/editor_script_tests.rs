//! Editor script ownership survives an unusable, partially typed template.
#![expect(clippy::expect_used, reason = "authored projection laws")]
#![expect(clippy::unwrap_used, reason = "authored projection laws")]

use super::*;
use serde_json::{Value, json};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::cstr;

const CONTROLS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/component-tag-classification/Controls.vue.txt"
));
const SPLIT: &str = "<!-- 😀 -->\n<script lang=\"ts\">\nexport const shared = 1;\n</script>\n<script setup lang=\"ts\">\nimport { defineComponent as build } from \"vue\";\nimport type { Component } from \"vue\";\nconst LocalCard: Component = build({});\n</script>\n<template>\n  😀 <Loc\n</template>\n";

fn generate(source: &str, editor: bool) -> GeneratedVueFile {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    generate_vue_virtual_ts(
        Path::new("App.vue"),
        source,
        &descriptor,
        &VirtualTsOptions::default(),
        VueCodegenOptions {
            check_options: Default::default(),
            typed_router_root: None,
            preserve_unused_diagnostics: false,
            options_api: false,
            preserve_authored_component: false,
            preserve_script_on_template_error: editor,
            component_name: None,
            preserve_event_navigation: true,
            legacy_vue2: false,
            dialect: Default::default(),
            template_syntax: Default::default(),
            experimental_in_tag_comments: false,
            experimental_patterned_template: false,
            experimental_strict_slot_children: false,
            hoist_shared_preamble: false,
            omit_vite_client_reference: false,
            runtime_prop_resolve_cache: None,
        },
    )
    .unwrap()
}

fn diagnostics(generated: &GeneratedVueFile) -> Vec<Value> {
    generated
        .diagnostics
        .iter()
        .map(|d| {
            json!({"file":d.file,"line":d.line,"column":d.column,
                "message":d.message,"code":d.code,"severity":d.severity,
                "blockType":d.block_type.map(|block| cstr!("{block:?}"))})
        })
        .collect()
}

#[test]
fn editor_preserves_whole_script_projection_maps_and_template_diagnostics() {
    for newline in ["\n", "\r\n"] {
        for source in [CONTROLS.replace("😀 <", "😀 <Loc"), SPLIT.into()] {
            let source = source.replace('\n', newline);
            let template = source.find("<template>").unwrap();
            let script = &source[..template];
            let editor = generate(&source, true);
            let script_only = generate(script, true);
            let batch = generate(&source, false);
            assert_ne!(editor.code, invalid_sfc_fallback_virtual_ts());
            assert_eq!(editor.code, script_only.code);
            assert_eq!(editor.mappings, script_only.mappings);
            assert_eq!(editor.semantic_links, script_only.semantic_links);
            assert_eq!(editor.typed_router_import, script_only.typed_router_import);
            assert_eq!(diagnostics(&editor), diagnostics(&batch));
            assert!(!editor.diagnostics.is_empty());
            assert!(editor.diagnostics.iter().all(|d| {
                d.block_type == Some(SfcBlockType::Template) && d.severity == 1 && d.code.is_none()
            }));
            assert!(!editor.mappings.is_empty());
            assert!(editor.mappings.iter().all(|m| m.src_range.end <= template));
            let module = source.find("\"vue\"").unwrap() + 1;
            assert!(editor.mappings.iter().any(|m| {
                m.src_range.start <= module
                    && m.src_range.end >= module + 3
                    && source.get(m.src_range.clone()) == editor.code.get(m.gen_range.clone())
            }));
            assert_eq!(batch.code, invalid_sfc_fallback_virtual_ts());
            assert!(batch.mappings.is_empty());
            assert!(batch.semantic_links.is_empty());
            let mapper =
                crate::batch::generate_vue_content_mapper_transform(Path::new("App.vue"), &source)
                    .unwrap();
            let mapper_script =
                crate::batch::generate_vue_content_mapper_transform(Path::new("App.vue"), script)
                    .unwrap();
            assert_eq!(mapper.text, mapper_script.text);
            assert_eq!(mapper.mappings, mapper_script.mappings);
            assert_eq!(mapper.semantic_links, mapper_script.semantic_links);
            assert!(!mapper.mappings.is_empty());
            assert!(!mapper.diagnostics.is_empty());
            assert!(
                mapper
                    .mappings
                    .iter()
                    .all(|mapping| { mapping.0[2] + mapping.0[3] <= template })
            );
        }
    }
}

#[test]
fn retained_typescript_syntax_has_only_the_native_diagnostic_owner() {
    let source = "<script setup lang=\"ts\">\nimport { ref } from \"vue\";\nconst Broken = ;\n</script>\n<template><Loc</template>\n";
    let editor = generate(source, true);
    let batch = generate(source, false);
    assert_ne!(editor.code, invalid_sfc_fallback_virtual_ts());
    let script_only = generate(&source[..source.find("<template>").unwrap()], true);
    assert_eq!(editor.code, script_only.code);
    assert_eq!(editor.mappings, script_only.mappings);
    assert_eq!(editor.semantic_links, script_only.semantic_links);
    assert!(
        editor
            .diagnostics
            .iter()
            .all(|d| d.block_type == Some(SfcBlockType::Template))
    );
    let batch_template = GeneratedVueFile {
        diagnostics: batch
            .diagnostics
            .iter()
            .filter(|d| d.block_type == Some(SfcBlockType::Template))
            .cloned()
            .collect(),
        ..batch
    };
    assert_eq!(diagnostics(&editor), diagnostics(&batch_template));
    assert!(editor.diagnostics.len() < generate(source, false).diagnostics.len());
}

#[test]
fn unusable_template_without_a_script_keeps_the_complete_fallback() {
    let source = "<template><Loc</template>\n";
    let editor = generate(source, true);
    let batch = generate(source, false);
    assert_eq!(editor.code, invalid_sfc_fallback_virtual_ts());
    assert_eq!(editor.code, batch.code);
    assert_eq!(editor.mappings, batch.mappings);
    assert_eq!(editor.semantic_links, batch.semantic_links);
    assert_eq!(diagnostics(&editor), diagnostics(&batch));
}

#[test]
fn complete_malformed_ts40_template_preserves_the_whole_original_fallback() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/davinci-ts40-projection/parse-recovery.vue"
    ));
    for newline in ["\n", "\r\n"] {
        let source = source.replace('\n', newline);
        let editor = generate(&source, true);
        let batch = generate(&source, false);
        assert_eq!(editor.code, invalid_sfc_fallback_virtual_ts());
        assert_eq!(editor.code, batch.code);
        assert_eq!(editor.mappings, batch.mappings);
        assert_eq!(editor.semantic_links, batch.semantic_links);
        assert_eq!(editor.typed_router_import, batch.typed_router_import);
        assert_eq!(diagnostics(&editor), diagnostics(&batch));
    }
}

#[test]
fn editor_documents_retain_diagnostic_only_inference_owners_through_jsx_prefixes() {
    use crate::batch::{
        ImportRewriter, VueDocumentVirtualTsOptions, generate_vue_document_virtual_ts_with_options,
    };
    use crate::virtual_ts::ProjectionFeatures;

    for language in ["ts", "tsx"] {
        let source = cstr!(
            "<script setup lang=\"{language}\">import Child from './Child.vue';</script>\n<template><Child :hidden=\"'wrong'\" @close=\"() => {{}}\" /></template>"
        );
        let document = generate_vue_document_virtual_ts_with_options(
            Path::new("App.vue"),
            &source,
            &VirtualTsOptions::default(),
            &ImportRewriter::new(),
            false,
            VueDocumentVirtualTsOptions {
                preserve_event_navigation: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(document.source_type.is_jsx(), language == "tsx");
        let inference = document
            .pre_rewrite_code
            .find("const __vize_emit_props_")
            .unwrap();
        let suffix = document.pre_rewrite_code.get(inference..).unwrap();
        let key = inference + suffix.find("\"hidden\"").unwrap();
        let value = inference + suffix.find("'wrong'").unwrap();
        let authored_key = source.find(":hidden").unwrap() + 1;
        let authored_value = source.find("'wrong'").unwrap();
        let row = document
            .mapping
            .rows()
            .find(|row| row.span.gen_range.start == key)
            .unwrap();
        assert_eq!(row.meta.features, ProjectionFeatures::DIAGNOSTICS);
        assert_eq!(
            document
                .mapping
                .diagnostic_range_to_authored(value, value + 7),
            Some((authored_value, authored_value + 7))
        );
        for feature in [
            ProjectionFeatures::HOVER,
            ProjectionFeatures::COMPLETION,
            ProjectionFeatures::DEFINITION,
        ] {
            let target = document
                .mapping
                .to_generated_for(authored_key, feature)
                .unwrap();
            assert!(!row.span.gen_range.contains(&target));
        }
        let mapper =
            crate::batch::generate_vue_content_mapper_transform(Path::new("App.vue"), &source)
                .unwrap();
        let inference = mapper.text.find("const __vize_emit_props_").unwrap();
        let suffix = mapper.text.get(inference..).unwrap();
        for offset in [
            inference + suffix.find("\"hidden\"").unwrap() + 1,
            inference + suffix.find("'wrong'").unwrap() + 1,
        ] {
            let spans = mapper
                .mappings
                .iter()
                .filter(|span| span.0[0] <= offset && offset < span.0[0] + span.0[1])
                .collect::<Vec<_>>();
            assert!(
                !spans.is_empty(),
                "inference geometry must remain available to diagnostics"
            );
            assert!(
                spans.iter().all(|span| span.0[5] == 0),
                "inference copies must not claim protocol editor operations"
            );
        }
        assert!(
            mapper.mappings.iter().any(|span| span.0[2] <= authored_key
                && authored_key < span.0[2] + span.0[3]
                && span.0[5] != 0),
            "the authored primary prop remains navigable"
        );
    }
}
