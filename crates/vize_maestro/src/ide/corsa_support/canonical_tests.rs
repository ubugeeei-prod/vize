use tower_lsp::lsp_types::Url;
use vize_canon::{LspLocation, LspPosition, LspRange};

use super::canonical::{
    CanonicalVirtualDocument, canonical_request_path, canonical_source_offset_to_position,
    map_canonical_corsa_location,
};
use super::request_file_uri;
use crate::ide::IdeContext;
use crate::server::ServerState;

fn canonical_doc(uri: &Url, source: &str) -> CanonicalVirtualDocument {
    let virtual_result =
        crate::ide::DiagnosticService::generate_virtual_ts(uri, source, false, false)
            .expect("virtual ts");
    CanonicalVirtualDocument {
        request_uri: request_file_uri(canonical_request_path(uri).as_str()),
        virtual_result,
        dependencies: Vec::new(),
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
