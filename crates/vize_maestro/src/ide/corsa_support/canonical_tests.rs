use tower_lsp::lsp_types::Url;
use vize_canon::{LspLocation, LspPosition, LspRange};

use super::canonical::{
    CanonicalMaterializedSource, CanonicalVirtualDocument, canonical_request_path,
    canonical_source_offset_to_position, map_canonical_corsa_location,
};
use super::request_file_uri;
use crate::ide::IdeContext;
use crate::server::ServerState;

fn canonical_doc(uri: &Url, source: &str) -> CanonicalVirtualDocument {
    let virtual_result =
        crate::ide::DiagnosticService::generate_virtual_ts(uri, source, false, false)
            .expect("virtual ts");
    CanonicalVirtualDocument {
        source_uri: uri.clone(),
        request_uri: request_file_uri(canonical_request_path(uri).as_str()),
        virtual_result,
        dependencies: Vec::new(),
        materialized_sources: Vec::new(),
        session_project_roots: Vec::new(),
    }
}

fn generated_offset_for_source(doc: &CanonicalVirtualDocument, source_offset: usize) -> usize {
    let (line, character) =
        canonical_source_offset_to_position(doc, source_offset).expect("mapped position");
    crate::ide::position_to_offset(&doc.virtual_result.code, line, character)
        .expect("generated offset")
}

#[test]
fn canonical_source_offset_maps_template_expression_to_generated_position() {
    let uri = Url::parse("file:///tmp/TypedTemplate.vue").expect("uri");
    let source = r#"<script setup lang="ts">
const user = { name: 'Ada' as string }
</script>

<template>
  {{ user.name }}
</template>
"#;
    let doc = canonical_doc(&uri, source);

    let source_offset = source.rfind("name").unwrap() + "na".len();
    let generated_offset = generated_offset_for_source(&doc, source_offset);
    let expected_offset = doc.virtual_result.code.find("user.name").unwrap() + "user.na".len();

    assert_eq!(generated_offset, expected_offset);
}

#[test]
fn canonical_source_offset_maps_bare_event_handler_to_handler_identifier() {
    let uri = Url::parse("file:///tmp/EventHandler.vue").expect("uri");
    let source = r#"<script setup lang="ts">
function submit() {}
</script>

<template>
  <button @click="submit">Save</button>
</template>
"#;
    let doc = canonical_doc(&uri, source);

    let source_offset = source.rfind("submit").unwrap() + "sub".len();
    let generated_offset = generated_offset_for_source(&doc, source_offset);
    let expected_offset = doc.virtual_result.code.find("((submit))").unwrap() + "((sub".len();

    assert_eq!(
        generated_offset, expected_offset,
        "event handler cursor should land on the generated handler expression:\n{}",
        doc.virtual_result.code
    );
}

#[test]
fn canonical_source_offset_maps_inline_event_handler_body_expression() {
    let uri = Url::parse("file:///tmp/InlineEventHandler.vue").expect("uri");
    let source = r#"<script setup lang="ts">
const user = { name: 'Ada' as string }
</script>

<template>
  <button @click="() => { user.name }">Save</button>
</template>
"#;
    let doc = canonical_doc(&uri, source);

    let source_offset = source.rfind("name").unwrap() + "na".len();
    let generated_offset = generated_offset_for_source(&doc, source_offset);
    let expected_offset = doc.virtual_result.code.rfind("user.name").unwrap() + "user.na".len();

    assert_eq!(
        generated_offset, expected_offset,
        "inline event handler cursor should land inside the generated callback expression:\n{}",
        doc.virtual_result.code
    );
}

#[test]
fn canonical_source_offset_maps_script_ts_usage_to_generated_position() {
    let uri = Url::parse("file:///tmp/ScriptUsage.vue").expect("uri");
    let source = r#"<script setup lang="ts">
const user = { name: 'Ada' as string }
const label = user.name
</script>
"#;
    let doc = canonical_doc(&uri, source);

    let source_offset = source.rfind("name").unwrap() + "na".len();
    let generated_offset = generated_offset_for_source(&doc, source_offset);
    let expected_offset = doc.virtual_result.code.rfind("user.name").unwrap() + "user.na".len();

    assert_eq!(generated_offset, expected_offset);
}

#[test]
fn canonical_location_maps_script_setup_class_member_implementation_target() {
    let uri = Url::parse("file:///tmp/Implementation.vue").expect("uri");
    let source = r#"<script setup lang="ts">
interface Formatter {
  format(value: string): string
}

class LabelFormatter implements Formatter {
  format(value: string): string {
    return value.toUpperCase()
  }
}
</script>
"#;
    let state = ServerState::new();
    let ctx = IdeContext::with_content(
        &state,
        &uri,
        source.find("Formatter").unwrap(),
        source.to_string(),
    );
    let doc = canonical_doc(&uri, source);
    let generated_start = doc
        .virtual_result
        .code
        .find("format(value: string): string {")
        .expect("generated class method");
    let generated_end = generated_start + "format".len();
    let (start_line, start_character) =
        crate::ide::offset_to_position(&doc.virtual_result.code, generated_start);
    let (end_line, end_character) =
        crate::ide::offset_to_position(&doc.virtual_result.code, generated_end);
    let location = LspLocation {
        uri: doc.request_uri.to_string(),
        range: LspRange {
            start: LspPosition {
                line: start_line,
                character: start_character,
            },
            end: LspPosition {
                line: end_line,
                character: end_character,
            },
        },
    };

    let mapped = map_canonical_corsa_location(&ctx, &doc, &location)
        .expect("implementation target should map to authored class method");
    let source_start = crate::ide::position_to_offset(
        source,
        mapped.range.start.line,
        mapped.range.start.character,
    )
    .expect("mapped start");
    let source_end =
        crate::ide::position_to_offset(source, mapped.range.end.line, mapped.range.end.character)
            .expect("mapped end");

    assert_eq!(&source[source_start..source_end], "format");
}

#[test]
fn canonical_source_offset_accounts_for_vue_import_rewrite_before_script_body() {
    let uri = Url::parse("file:///tmp/Parent.vue").expect("uri");
    let source = r#"<script setup lang="ts">
import Child from "./Child.vue";
const selected = Child;
</script>
"#;
    let doc = canonical_doc(&uri, source);

    let source_offset = source.rfind("Child").unwrap() + "Ch".len();
    let generated_offset = generated_offset_for_source(&doc, source_offset);
    let expected_offset = doc.virtual_result.code.rfind("Child").unwrap() + "Ch".len();

    assert_eq!(generated_offset, expected_offset);
}

#[test]
fn canonical_location_rejects_deleted_files_but_keeps_open_unsaved_files() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let importer_uri = Url::from_file_path(workspace.path().join("Importer.vue")).expect("URI");
    let target_uri = Url::from_file_path(workspace.path().join("Deleted.vue")).expect("URI");
    let source = "<template><Deleted /></template>\n";
    let state = ServerState::new();
    let ctx = IdeContext::with_content(&state, &importer_uri, 11, source.to_string());
    let doc = canonical_doc(&importer_uri, source);
    let location = LspLocation {
        uri: target_uri.to_string(),
        range: LspRange {
            start: LspPosition {
                line: 0,
                character: 0,
            },
            end: LspPosition {
                line: 0,
                character: 0,
            },
        },
    };

    assert!(map_canonical_corsa_location(&ctx, &doc, &location).is_none());

    state.documents.open(
        target_uri.clone(),
        "<template />\n".to_string(),
        1,
        "vue".to_string(),
    );
    assert_eq!(
        map_canonical_corsa_location(&ctx, &doc, &location)
            .expect("open unsaved target")
            .uri,
        target_uri
    );
}

#[test]
fn canonical_location_maps_exact_package_shadow_and_rejects_synthetic_coordinates() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let importer_uri = Url::from_file_path(workspace.path().join("Importer.vue")).expect("URI");
    let authored_path = workspace.path().join("packages/ui/Widget.vue");
    std::fs::create_dir_all(authored_path.parent().unwrap()).unwrap();
    std::fs::write(&authored_path, "export const alpha = 1;\n").unwrap();
    let authored_uri = Url::from_file_path(&authored_path).unwrap();
    let shadow_path = workspace
        .path()
        .join("node_modules/.vize/canon/Widget.vue.ts");
    std::fs::create_dir_all(shadow_path.parent().unwrap()).unwrap();
    std::fs::write(&shadow_path, "export const alpha = 1;\n").unwrap();
    let shadow_uri = Url::from_file_path(&shadow_path).unwrap();
    let source = "<template />\n";
    let state = ServerState::new();
    let ctx = IdeContext::with_content(&state, &importer_uri, 0, source.to_string());
    let mut doc = canonical_doc(&importer_uri, source);
    doc.materialized_sources.push(CanonicalMaterializedSource {
        source_uri: authored_uri.clone(),
        source: "export const alpha = 1;\n".into(),
        request_uri: shadow_uri.to_string().into(),
        virtual_result: crate::ide::diagnostics::VirtualTsResult {
            code: "export const alpha = 1;\n".to_string(),
            source_mappings: Vec::new(),
            semantic_links: Vec::new(),
            import_source_map: vize_canon::ImportSourceMap::empty(),
            user_code_start_line: 0,
            sfc_script_start_line: 0,
            template_scope_start_line: 0,
            line_mappings: Vec::new(),
            skipped_import_lines: 0,
        },
        mapping_kind: vize_canon::CorsaMaterializedMappingKind::AuthoredIdentity,
    });
    let location = LspLocation {
        uri: shadow_uri.to_string(),
        range: LspRange {
            start: LspPosition {
                line: 0,
                character: 13,
            },
            end: LspPosition {
                line: 0,
                character: 18,
            },
        },
    };
    let mapped = map_canonical_corsa_location(&ctx, &doc, &location).expect("authored shadow");
    assert_eq!(mapped.uri, authored_uri);
    assert_eq!(mapped.range.start.character, 13);
    assert_eq!(mapped.range.end.character, 18);
    assert!(!mapped.uri.path().contains(".vize"));

    doc.materialized_sources[0].mapping_kind = vize_canon::CorsaMaterializedMappingKind::Synthetic;
    assert!(
        map_canonical_corsa_location(&ctx, &doc, &location).is_none(),
        "synthetic forwarder coordinates must fail closed, not leak a Canon URI"
    );

    let private_root = workspace.path().join("private-session");
    let unknown = private_root.join("__vize_missing_module.d.ts");
    std::fs::create_dir_all(&private_root).unwrap();
    std::fs::write(&unknown, "declare const hidden: any;\n").unwrap();
    doc.session_project_roots = vec![private_root];
    let unknown = LspLocation {
        uri: Url::from_file_path(unknown).unwrap().to_string(),
        range: location.range,
    };
    assert!(
        map_canonical_corsa_location(&ctx, &doc, &unknown).is_none(),
        "unindexed files under the private Canon session must never leak"
    );
}
