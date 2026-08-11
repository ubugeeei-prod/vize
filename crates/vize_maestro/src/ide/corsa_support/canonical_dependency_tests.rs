use tower_lsp::lsp_types::Url;
use vize_canon::{LspLocation, LspPosition, LspRange};

use super::canonical::{
    CanonicalDependencyDocument, CanonicalVirtualDocument, canonical_request_path,
    map_canonical_corsa_location,
};
use super::request_file_uri;
use crate::ide::{DiagnosticService, IdeContext, offset_to_position, position_to_offset};
use crate::server::ServerState;

pub(super) fn mapped_document(uri: &Url, source: &str) -> CanonicalDependencyDocument {
    CanonicalDependencyDocument {
        source_uri: uri.clone(),
        source: source.into(),
        request_uri: request_file_uri(canonical_request_path(uri).as_str()),
        virtual_result: DiagnosticService::generate_virtual_ts(uri, source, false, false)
            .expect("virtual ts"),
    }
}

pub(super) fn host_document(uri: &Url, source: &str) -> CanonicalVirtualDocument {
    CanonicalVirtualDocument {
        source_uri: uri.clone(),
        request_uri: request_file_uri(canonical_request_path(uri).as_str()),
        virtual_result: DiagnosticService::generate_virtual_ts(uri, source, false, false)
            .expect("virtual ts"),
        dependencies: Vec::new(),
        materialized_sources: Vec::new(),
        session_project_roots: Vec::new(),
    }
}

fn lsp_range(content: &str, start: usize, end: usize) -> LspRange {
    let (start_line, start_character) = offset_to_position(content, start);
    let (end_line, end_character) = offset_to_position(content, end);
    LspRange {
        start: LspPosition {
            line: start_line,
            character: start_character,
        },
        end: LspPosition {
            line: end_line,
            character: end_character,
        },
    }
}

#[test]
fn maps_dependency_locations_with_exact_uri_utf16_and_import_rewrite() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host View.vue");
    let child_path = project.path().join("Child View.vue");
    let host_uri = Url::from_file_path(&host_path).expect("host uri");
    let child_uri = Url::from_file_path(&child_path).expect("child uri");
    let host_source = "<script setup lang=\"ts\">import Child from \"./Child View.vue\"</script>";
    let child_source = r#"<script setup lang="ts">
import Leaf from "./Leaf.vue";
const emoji = "💥"; const childValue = 1;
</script>
<template>{{ childValue }}{{ emoji }}<Leaf /></template>"#;
    std::fs::write(&host_path, host_source).expect("host");
    std::fs::write(&child_path, child_source).expect("child");

    let state = ServerState::new();
    state.documents.open(
        host_uri.clone(),
        host_source.to_string(),
        1,
        "vue".to_string(),
    );
    let ctx = IdeContext::new(&state, &host_uri, host_source.find("Child").unwrap())
        .expect("host context");
    let dependency = mapped_document(&child_uri, child_source);
    let source_start = child_source.rfind("childValue").expect("authored token");
    let source_end = source_start + "childValue".len();
    let generated_start = generated_offset_for_source(&dependency, source_start);
    let generated_end = generated_offset_for_source(&dependency, source_end);
    let backend_location = LspLocation {
        uri: dependency.request_uri.to_string(),
        range: lsp_range(
            &dependency.virtual_result.code,
            generated_start,
            generated_end,
        ),
    };
    let mut host = host_document(&host_uri, host_source);
    host.dependencies.push(dependency);

    let mapped = map_canonical_corsa_location(&ctx, &host, &backend_location)
        .expect("authored dependency location");
    assert_eq!(mapped.uri, child_uri, "percent-encoded URI must round-trip");
    let mapped_source_start = position_to_offset(
        child_source,
        mapped.range.start.line,
        mapped.range.start.character,
    )
    .expect("source start");
    let mapped_source_end = position_to_offset(
        child_source,
        mapped.range.end.line,
        mapped.range.end.character,
    )
    .expect("source end");
    assert_eq!(
        &child_source[mapped_source_start..mapped_source_end],
        "childValue"
    );
}

#[test]
fn rejects_unknown_synthetic_vue_locations_but_preserves_real_vue_ts_files() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host.vue");
    let missing_source_path = project.path().join("Missing.vue");
    let real_ts_path = project.path().join("Actual.vue.ts");
    let real_vue_path = project.path().join("Actual.vue");
    let host_uri = Url::from_file_path(&host_path).expect("host uri");
    let host_source = "<template><main /></template>";
    std::fs::write(&host_path, host_source).expect("host");
    std::fs::write(&missing_source_path, "<template><div /></template>").expect("missing source");
    std::fs::write(&real_ts_path, "export const real = true;\n").expect("real ts");
    std::fs::write(&real_vue_path, "<template><aside /></template>").expect("real vue");

    let state = ServerState::new();
    state.documents.open(
        host_uri.clone(),
        host_source.to_string(),
        1,
        "vue".to_string(),
    );
    let ctx = IdeContext::new(&state, &host_uri, 1).expect("host context");
    let host = host_document(&host_uri, host_source);
    let range = LspRange {
        start: LspPosition {
            line: 0,
            character: 0,
        },
        end: LspPosition {
            line: 0,
            character: 4,
        },
    };
    let missing = LspLocation {
        uri: path_uri(&missing_source_path.with_extension("vue.ts")),
        range: range.clone(),
    };
    assert!(
        map_canonical_corsa_location(&ctx, &host, &missing).is_none(),
        "canonical synthetic locations without retained metadata must fail closed",
    );

    let real_ts_uri = Url::from_file_path(&real_ts_path).expect("real ts uri");
    let real = LspLocation {
        uri: real_ts_uri.to_string(),
        range,
    };
    let mapped = map_canonical_corsa_location(&ctx, &host, &real).expect("real TS location");
    assert_eq!(mapped.uri, real_ts_uri);
    assert_eq!(mapped.range.start.character, 0);
    assert_eq!(mapped.range.end.character, 4);
}

fn path_uri(path: &std::path::Path) -> String {
    Url::from_file_path(path).expect("file uri").to_string()
}

pub(super) fn generated_offset_for_source(
    document: &CanonicalDependencyDocument,
    source_offset: usize,
) -> usize {
    let mapping = document
        .virtual_result
        .source_mappings
        .iter()
        .filter(|mapping| {
            source_offset >= mapping.src_range.start && source_offset <= mapping.src_range.end
        })
        .min_by_key(|mapping| mapping.src_range.end - mapping.src_range.start)
        .expect("source mapping");
    let generated_pre_rewrite =
        if let Some(span) = mapping.sub_spans.iter().find(|span| {
            source_offset >= span.src_range.start && source_offset <= span.src_range.end
        }) {
            span.gen_range.start
                + source_offset
                    .saturating_sub(span.src_range.start)
                    .min(span.gen_range.end - span.gen_range.start)
        } else {
            mapping.gen_range.start
                + source_offset
                    .saturating_sub(mapping.src_range.start)
                    .min(mapping.gen_range.end - mapping.gen_range.start)
        };
    document
        .virtual_result
        .import_source_map
        .get_virtual_offset(generated_pre_rewrite as u32) as usize
}
