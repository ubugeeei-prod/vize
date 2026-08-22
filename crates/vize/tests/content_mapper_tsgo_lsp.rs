use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use corsa_lsp::{LspClient, LspSpawnConfig, VirtualDocument, jsonrpc::InboundEvent};
use lsp_types::{FileChangeType, Uri};
use serde_json::json;

mod content_mapper_lsp_support;
use content_mapper_lsp_support::raw_requests::{
    RawInitialize, RawInitialized, RawSetContentMapperContributions, RawSignatureHelp,
};
use content_mapper_lsp_support::{
    EditorResponder, assert_component_completions, assert_component_members,
    assert_component_navigation, assert_no_generated_uri_or_zero_range, completion,
    contains_location, contains_range, copy_fixture, definition, document_highlights,
    editor_capabilities, file_uri, hover, install_packages, notify_file_changes, position,
    pull_diagnostics, references, rename, try_pull_diagnostics, workspace_root,
};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";

struct StopOnDrop<'a>(&'a AtomicBool);

impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

#[test]
fn standard_tsgo_lsp_maps_core_symbol_features_to_authored_vue() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper LSP conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-lsp-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());

    let child_path = project.path().join("src/Child.vue");
    let source = std::fs::read_to_string(&child_path).unwrap();
    let child_uri = file_uri(&child_path);
    let root_uri = file_uri(project.path());
    let symbol_offset = source.rfind("count.toFixed").unwrap();
    let symbol_position = position(&source, symbol_offset + 1);
    let completion_position = position(&source, symbol_offset + "count.".len());
    let signature_position = position(&source, symbol_offset + "count.toFixed(".len());
    let declaration_offset = source.find("count: number").unwrap();
    let declaration_position = position(&source, declaration_offset);
    let app_path = project.path().join("src/App.vue");
    let app_source = std::fs::read_to_string(&app_path).unwrap();
    let app_uri = file_uri(&app_path);
    let component_position = position(&app_source, app_source.find("<Child").unwrap() + 1);

    let stop = AtomicBool::new(false);
    let editor = EditorResponder::default();
    std::thread::scope(|scope| {
        let _stop_on_drop = StopOnDrop(&stop);
        corsa::runtime::block_on(async {
            let client = LspClient::spawn(
                LspSpawnConfig::new(&tsgo)
                    .with_cwd(project.path())
                    .with_request_timeout(Some(Duration::from_secs(30))),
            )
            .await
            .unwrap();
            let responder_client = client.clone();
            let events = responder_client.subscribe();
            let stop_ref = &stop;
            let editor_ref = &editor;
            let responder = scope.spawn(move || {
                while !stop_ref.load(Ordering::Relaxed) {
                    if let Ok(InboundEvent::Request { id, method, params }) =
                        events.recv_timeout(Duration::from_millis(50))
                    {
                        let result = editor_ref.respond_to(method.as_str(), &params);
                        let _ = responder_client.respond(id, result);
                    }
                }
            });
            let initialize = client
                .request::<RawInitialize>(json!({
                    "processId": std::process::id(),
                    "rootUri": root_uri,
                    "workspaceFolders": [{ "uri": root_uri, "name": "content-mapper-lsp" }],
                    "capabilities": editor_capabilities(),
                    "initializationOptions": { "runExternalCode": true }
                }))
                .await
                .unwrap();
            assert!(initialize["capabilities"].is_object(), "{initialize:#}");
            client.notify::<RawInitialized>(json!({})).unwrap();

            let contributed = client
                .request::<RawSetContentMapperContributions>(json!({
                    "contributions": [{
                        "contributorId": "vize",
                        "extensions": [".vue"],
                        "inferredProjectContribution": {
                            "options": {},
                            "manifest": {
                                "name": "vize",
                                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                                "compilerOptions": ["noUnusedLocals"]
                            }
                        }
                    }],
                    "openDocuments": [{ "uri": child_uri }, { "uri": app_uri }]
                }))
                .await
                .unwrap();
            assert!(contributed.is_null(), "{contributed:#}");
            editor.assert_vue_did_open_registration();

            let uri = Uri::from_str(&child_uri).unwrap();
            let overlay = client.overlay();
            overlay
                .open(VirtualDocument::new(uri.clone(), "vue", source.as_str()))
                .unwrap();
            let app_document_uri = Uri::from_str(&app_uri).unwrap();
            overlay
                .open(VirtualDocument::new(
                    app_document_uri.clone(),
                    "vue",
                    app_source.as_str(),
                ))
                .unwrap();
            assert_component_navigation(
                &client,
                &app_uri,
                &component_position,
                "Child",
                &child_uri,
            )
            .await;

            assert_component_members(&client, (&app_uri, &app_source), (&child_uri, &source)).await;

            assert_component_completions(&client, &overlay, &app_document_uri, &app_source).await;

            let completion = completion(&client, &child_uri, &completion_position).await;
            assert!(
                serde_json::to_string(&completion)
                    .unwrap()
                    .contains("toFixed"),
                "{completion:#}"
            );

            let signature = client
                .request::<RawSignatureHelp>(json!({
                    "textDocument": { "uri": child_uri },
                    "position": signature_position
                }))
                .await
                .unwrap();
            let signature_text = serde_json::to_string(&signature).unwrap();
            assert!(
                signature_text.contains("fractionDigits") && signature_text.contains("number"),
                "{signature:#}"
            );

            let hover = hover(&client, &child_uri, &symbol_position).await;
            let hover_text = serde_json::to_string(&hover).unwrap();
            assert!(
                hover_text.contains("count") && hover_text.contains("number"),
                "{hover:#}"
            );

            let definition = definition(&client, &child_uri, &symbol_position).await;
            assert_no_generated_uri_or_zero_range(&definition);
            let definition_text = serde_json::to_string(&definition).unwrap();
            assert!(
                definition_text.contains(child_uri.as_str()),
                "{definition:#}"
            );
            assert!(
                contains_location(&definition, &child_uri, &declaration_position),
                "{definition:#}"
            );
            assert!(!definition_text.contains(".vue.ts"), "{definition:#}");

            let references = references(&client, &child_uri, &symbol_position).await;
            assert_no_generated_uri_or_zero_range(&references);
            let references_text = serde_json::to_string(&references).unwrap();
            assert!(
                references_text.matches(child_uri.as_str()).count() >= 2,
                "{references:#}"
            );
            assert!(!references_text.contains(".vue.ts"), "{references:#}");

            let declaration_end = position(&source, declaration_offset + "count".len());
            let usage_start = position(&source, symbol_offset);
            let usage_end = position(&source, symbol_offset + "count".len());
            let highlights = document_highlights(&client, &child_uri, &symbol_position).await;
            assert_no_generated_uri_or_zero_range(&highlights);
            assert!(
                contains_range(&highlights, &declaration_position, &declaration_end),
                "{highlights:#}"
            );
            assert!(
                contains_range(&highlights, &usage_start, &usage_end),
                "{highlights:#}"
            );

            let rename = rename(&client, &child_uri, &symbol_position, "renamedCount").await;
            assert_no_generated_uri_or_zero_range(&rename);
            let rename_text = serde_json::to_string(&rename).unwrap();
            assert!(rename_text.contains(child_uri.as_str()), "{rename:#}");
            assert!(
                rename_text.matches("renamedCount").count() >= 2,
                "{rename:#}"
            );
            assert!(!rename_text.contains(".vue.ts"), "{rename:#}");

            let clean = pull_diagnostics(&client, &child_uri).await;
            assert_eq!(clean["items"], json!([]), "{clean:#}");

            let broken_source = source.replace("count.toFixed(0)", "count.missing()");
            overlay.replace(&uri, broken_source.as_str()).unwrap();
            let broken = pull_diagnostics(&client, &child_uri).await;
            let broken_text = serde_json::to_string(&broken).unwrap();
            assert!(
                broken_text.contains("2339") && broken_text.contains("missing"),
                "{broken:#}"
            );
            let missing = position(&broken_source, broken_source.find("missing").unwrap());
            assert_eq!(broken["items"][0]["range"]["start"], missing, "{broken:#}");

            overlay.replace(&uri, source.as_str()).unwrap();
            let repaired = pull_diagnostics(&client, &child_uri).await;
            assert_eq!(repaired["items"], json!([]), "{repaired:#}");

            overlay.replace(&uri, broken_source.as_str()).unwrap();
            let dirty = pull_diagnostics(&client, &child_uri).await;
            assert!(!dirty["items"].as_array().unwrap().is_empty(), "{dirty:#}");

            assert!(overlay.close(&uri).unwrap().is_some());
            let closed = pull_diagnostics(&client, &child_uri).await;
            assert_eq!(closed["items"], json!([]), "{closed:#}");

            let created_path = project.path().join("src/Created.vue");
            let created_source = r#"<script setup lang="ts">
const value = 1;
</script>
<template>{{ value.missing() }}</template>
"#;
            std::fs::write(&created_path, created_source).unwrap();
            let created_uri = file_uri(&created_path);
            notify_file_changes(&client, &[(created_uri.as_str(), FileChangeType::CREATED)]);
            let unopened_created = pull_diagnostics(&client, &created_uri).await;
            assert!(
                serde_json::to_string(&unopened_created)
                    .unwrap()
                    .contains("2339"),
                "{unopened_created:#}"
            );
            let created_document_uri = Uri::from_str(&created_uri).unwrap();
            overlay
                .open(VirtualDocument::new(
                    created_document_uri.clone(),
                    "vue",
                    created_source,
                ))
                .unwrap();
            let created = pull_diagnostics(&client, &created_uri).await;
            assert!(
                serde_json::to_string(&created).unwrap().contains("2339"),
                "{created:#}"
            );
            let missing = position(created_source, created_source.find("missing").unwrap());
            assert_eq!(
                created["items"][0]["range"]["start"], missing,
                "{created:#}"
            );

            let renamed_path = project.path().join("src/Renamed.vue");
            assert!(overlay.close(&created_document_uri).unwrap().is_some());
            std::fs::rename(&created_path, &renamed_path).unwrap();
            let renamed_uri = file_uri(&renamed_path);
            notify_file_changes(
                &client,
                &[
                    (created_uri.as_str(), FileChangeType::DELETED),
                    (renamed_uri.as_str(), FileChangeType::CREATED),
                ],
            );
            let unopened_renamed = pull_diagnostics(&client, &renamed_uri).await;
            assert!(
                serde_json::to_string(&unopened_renamed)
                    .unwrap()
                    .contains("2339"),
                "{unopened_renamed:#}"
            );
            let renamed_document_uri = Uri::from_str(&renamed_uri).unwrap();
            overlay
                .open(VirtualDocument::new(
                    renamed_document_uri.clone(),
                    "vue",
                    created_source,
                ))
                .unwrap();
            let renamed = pull_diagnostics(&client, &renamed_uri).await;
            assert!(
                serde_json::to_string(&renamed).unwrap().contains("2339"),
                "{renamed:#}"
            );
            assert!(try_pull_diagnostics(&client, &created_uri).await.is_err());

            assert!(overlay.close(&renamed_document_uri).unwrap().is_some());
            std::fs::remove_file(&renamed_path).unwrap();
            notify_file_changes(&client, &[(renamed_uri.as_str(), FileChangeType::DELETED)]);
            assert!(try_pull_diagnostics(&client, &renamed_uri).await.is_err());
            stop.store(true, Ordering::Relaxed);
            client.graceful_close().await.unwrap();
            responder.join().unwrap();
        });
    });
}
