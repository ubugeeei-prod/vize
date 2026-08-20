use super::*;
use tower_lsp::{
    LspService,
    lsp_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, TextDocumentContentChangeEvent, TextDocumentIdentifier,
        TextDocumentItem, VersionedTextDocumentIdentifier,
    },
};
use vize_carton::cstr;

fn quiet_service() -> tower_lsp::LspService<MaestroServer> {
    let (service, _socket) = LspService::new(MaestroServer::new);
    service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({
            "lint": false,
            "typecheck": false,
            "ecosystem": false
        })));
    service
}

fn uri(path: &str) -> Url {
    Url::parse(&cstr!("file:///{path}")).unwrap()
}

fn open_vue(server: &MaestroServer, uri: Url, text: &str, version: i32) {
    futures::executor::block_on(server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri,
            language_id: "vue".to_string(),
            version,
            text: text.to_string(),
        },
    }));
}

fn open_art(server: &MaestroServer, uri: Url, text: &str, version: i32) {
    futures::executor::block_on(server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri,
            language_id: "art-vue".to_string(),
            version,
            text: text.to_string(),
        },
    }));
}

fn full_change(uri: Url, version: i32, text: &str) -> DidChangeTextDocumentParams {
    DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier { uri, version },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: text.to_string(),
        }],
    }
}

fn replace_text_change(
    uri: Url,
    version: i32,
    source: &str,
    needle: &str,
    replacement: &str,
) -> DidChangeTextDocumentParams {
    let offset = source.find(needle).unwrap();
    let (line, character) = crate::ide::offset_to_position(source, offset);
    DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier { uri, version },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: Some(Range::new(
                Position::new(line, character),
                Position::new(line, character + needle.len() as u32),
            )),
            range_length: None,
            text: replacement.to_string(),
        }],
    }
}

#[test]
fn did_open_stores_document_and_generates_virtual_docs() {
    let service = quiet_service();
    let server = service.inner();
    let uri = uri("Open.vue");
    let source = "<script setup lang=\"ts\">\nconst message = 'hi'\n</script>\n\
                  <template>\n  <div>{{ message }}</div>\n</template>\n";

    open_vue(server, uri.clone(), source, 7);

    let doc = server.state.documents.get(&uri).unwrap();
    assert_eq!(doc.version, 7);
    assert_eq!(doc.language_id, "vue");
    assert_eq!(doc.text(), source);

    let virtual_docs = server.state.get_virtual_docs(&uri).unwrap();
    assert!(virtual_docs.template.is_some());
    assert!(virtual_docs.script_setup.is_some());
}

#[test]
fn did_change_full_content_replaces_document_and_rebuilds_virtual_docs() {
    let service = quiet_service();
    let server = service.inner();
    let uri = uri("Changed.vue");
    let before = "<template>{{ beforeMessage }}</template>\n";
    let after = "<template>{{ afterMessage }}</template>\n";

    open_vue(server, uri.clone(), before, 1);
    futures::executor::block_on(server.did_change(full_change(uri.clone(), 2, after)));

    let doc = server.state.documents.get(&uri).unwrap();
    assert_eq!(doc.version, 2);
    assert_eq!(doc.text(), after);

    let template = server
        .state
        .get_virtual_docs(&uri)
        .and_then(|docs| docs.template.clone())
        .expect("template virtual doc should be regenerated");
    assert!(template.content.contains("afterMessage"));
    assert!(!template.content.contains("beforeMessage"));
}

#[test]
fn did_change_incremental_content_updates_document_and_virtual_docs() {
    let service = quiet_service();
    let server = service.inner();
    let uri = uri("Incremental.vue");
    let before = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n\
                  <template>{{ count }}</template>\n";

    open_vue(server, uri.clone(), before, 1);
    futures::executor::block_on(server.did_change(replace_text_change(
        uri.clone(),
        2,
        before,
        "1",
        "2",
    )));

    let doc = server.state.documents.get(&uri).unwrap();
    assert_eq!(doc.version, 2);
    assert!(doc.text().contains("count = 2"));

    let script_setup = server
        .state
        .get_virtual_docs(&uri)
        .and_then(|docs| docs.script_setup.clone())
        .expect("script setup virtual doc should be regenerated");
    assert!(script_setup.content.contains("count = 2"));
}

#[test]
fn did_change_rebuilds_cached_art_variant_docs_after_create_edit_rename_delete() {
    let service = quiet_service();
    let server = service.inner();
    let uri = uri("Lifecycle.art.vue");
    let before = r#"<script setup lang="ts">
const primaryLabel = "primary"
const secondaryLabel = "secondary"
const editedLabel = "edited"
</script>

<art title="Lifecycle">
  <variant name="Primary" default>
    <p>{{ primaryLabel }}</p>
  </variant>
</art>
"#;
    let created = before.replace(
        "</art>",
        "  <variant name=\"Secondary\">\n    <p>{{ secondaryLabel }}</p>\n  </variant>\n</art>",
    );
    let edited = created.replace("{{ secondaryLabel }}", "{{ editedLabel }}");
    let renamed = edited.replace("name=\"Secondary\"", "name=\"Renamed\"");
    let deleted = renamed.replace(
        "  <variant name=\"Renamed\">\n    <p>{{ editedLabel }}</p>\n  </variant>\n",
        "",
    );

    open_art(server, uri.clone(), before, 1);
    let docs = server.state.get_virtual_docs(&uri).unwrap();
    assert_eq!(docs.art_templates.len(), 1);
    assert!(
        docs.art_template(0)
            .unwrap()
            .content
            .contains("primaryLabel")
    );

    futures::executor::block_on(server.did_change(full_change(uri.clone(), 2, &created)));
    let docs = server.state.get_virtual_docs(&uri).unwrap();
    assert_eq!(docs.art_templates.len(), 2);
    assert!(
        docs.art_template(1)
            .unwrap()
            .content
            .contains("secondaryLabel")
    );

    futures::executor::block_on(server.did_change(full_change(uri.clone(), 3, &edited)));
    let docs = server.state.get_virtual_docs(&uri).unwrap();
    let edited_template = &docs.art_template(1).unwrap().content;
    assert!(
        edited_template.contains("= editedLabel;"),
        "edited variant expression was not rebuilt:\n{edited_template}",
    );
    assert_eq!(
        edited_template.matches("secondaryLabel").count(),
        1,
        "the stale secondaryLabel expression should be gone while the setup declaration remains:\n{edited_template}",
    );

    futures::executor::block_on(server.did_change(full_change(uri.clone(), 4, &renamed)));
    let docs = server.state.get_virtual_docs(&uri).unwrap();
    assert_eq!(docs.art_templates.len(), 2);
    let renamed_template = docs.art_template(1).unwrap();
    assert!(renamed_template.content.contains("editedLabel"));
    let source_offset = renamed.rfind("editedLabel").unwrap();
    let generated_offset = renamed_template
        .source_map
        .to_generated(source_offset as u32)
        .expect("renamed variant mapping") as usize;
    assert_eq!(
        &renamed_template.content[generated_offset..generated_offset + "editedLabel".len()],
        "editedLabel",
        "the renamed variant mapping should follow the shifted authored offset:\n{}",
        renamed_template.content,
    );

    futures::executor::block_on(server.did_change(full_change(uri.clone(), 5, &deleted)));
    let docs = server.state.get_virtual_docs(&uri).unwrap();
    assert_eq!(docs.art_templates.len(), 1);
    assert!(docs.art_template(1).is_none());
    assert!(
        !docs
            .art_template(0)
            .unwrap()
            .content
            .contains("= editedLabel;")
    );
}

#[test]
fn did_change_unopened_document_does_not_create_document_or_virtual_docs() {
    let service = quiet_service();
    let server = service.inner();
    let uri = uri("Missing.vue");

    futures::executor::block_on(server.did_change(full_change(
        uri.clone(),
        2,
        "<template>ignored</template>",
    )));

    assert!(!server.state.documents.contains(&uri));
    assert!(server.state.get_virtual_docs(&uri).is_none());
}

#[test]
fn did_close_removes_document_and_virtual_docs() {
    let service = quiet_service();
    let server = service.inner();
    let uri = uri("Close.vue");
    let source = "<template>{{ message }}</template>\n";

    open_vue(server, uri.clone(), source, 1);
    assert!(server.state.documents.contains(&uri));
    assert!(server.state.get_virtual_docs(&uri).is_some());

    futures::executor::block_on(server.did_close(DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
    }));

    assert!(!server.state.documents.contains(&uri));
    assert!(server.state.get_virtual_docs(&uri).is_none());
}

#[test]
fn did_save_unopened_document_does_not_create_document() {
    let service = quiet_service();
    let server = service.inner();
    let uri = uri("SaveOnly.vue");

    futures::executor::block_on(server.did_save(DidSaveTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        text: Some("<template>ignored</template>".to_string()),
    }));

    assert!(!server.state.documents.contains(&uri));
    assert!(server.state.get_virtual_docs(&uri).is_none());
}
