use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use corsa_lsp::{LspClient, LspSpawnConfig, VirtualDocument, jsonrpc::InboundEvent};
use lsp_types::Uri;
use serde_json::{Value, json};
use vize_s0::FxHashSet;

#[allow(dead_code)]
#[path = "content_mapper_tsgo_lsp_event_forms/cases.rs"]
mod cases;
use cases::EVENT_CASES;
#[path = "content_mapper_tsgo_lsp_event_forms/model_props.rs"]
mod model_props;

mod content_mapper_lsp_support;
use content_mapper_lsp_support::{
    EditorResponder, assert_completion, assert_no_generated_uri_or_zero_range,
    assert_prop_navigation, contains_location, contains_location_range, contains_text_edit,
    copy_fixture, editor_capabilities, file_uri, install_packages, position, pull_diagnostics,
    references, rename, workspace_root,
};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";

struct StopOnDrop<'a>(&'a AtomicBool);
impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}
struct RawInitialize;
struct RawSetContentMapperContributions;
struct RawInitialized;
impl lsp_types::request::Request for RawInitialize {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "initialize";
}

impl lsp_types::request::Request for RawSetContentMapperContributions {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "custom/setContentMapperContributions";
}

impl lsp_types::notification::Notification for RawInitialized {
    type Params = Value;
    const METHOD: &'static str = "initialized";
}

#[test]
fn standard_tsgo_lsp_maps_event_symbol_navigation() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper LSP conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-lsp-event-forms-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());

    let app_path = project.path().join("src/App.vue");
    let app_source = std::fs::read_to_string(&app_path).unwrap();
    let app_uri = file_uri(&app_path);
    let cases = EVENT_CASES.map(
        |(
            file,
            usage,
            declaration,
            name,
            ty,
            renamed,
            reference_shift,
            reference_len,
            rename_supported,
        )| {
            let path = project.path().join("src").join(file);
            let source = std::fs::read_to_string(&path).unwrap();
            let uri = file_uri(&path);
            let usage_position = position(&app_source, app_source.find(usage).unwrap() + 1);
            let usage_end = position(&app_source, app_source.find(usage).unwrap() + usage.len());
            let declaration_position = position(&source, source.find(declaration).unwrap());
            let reference_position =
                position(&source, source.find(declaration).unwrap() + reference_shift);
            let reference_end = position(
                &source,
                source.find(declaration).unwrap() + reference_shift + reference_len,
            );
            (
                uri,
                source,
                usage_position,
                usage_end,
                declaration_position,
                reference_position,
                reference_end,
                name,
                ty,
                renamed,
                rename_supported,
            )
        },
    );
    let root_uri = file_uri(project.path());

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
                    "workspaceFolders": [{ "uri": root_uri, "name": "event-forms" }],
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
                    "openDocuments": std::iter::once(json!({ "uri": &app_uri }))
                        .chain(cases.iter().map(|case| json!({ "uri": case.0 })))
                        .collect::<Vec<_>>()
                }))
                .await
                .unwrap();
            assert!(contributed.is_null(), "{contributed:#}");
            editor.assert_vue_did_open_registration();

            let overlay = client.overlay();
            let app_document_uri = Uri::from_str(&app_uri).unwrap();
            overlay
                .open(VirtualDocument::new(
                    app_document_uri.clone(),
                    "vue",
                    app_source.as_str(),
                ))
                .unwrap();
            let mut opened = FxHashSet::default();
            let zero = json!({ "line": 0, "character": 0 });
            for (uri, source, ..) in &cases {
                if opened.insert(uri.as_str()) {
                    overlay
                        .open(VirtualDocument::new(
                            Uri::from_str(uri).unwrap(),
                            "vue",
                            source.as_str(),
                        ))
                        .unwrap();
                }
            }
            let app_clean = pull_diagnostics(&client, &app_uri).await;
            assert_eq!(app_clean["items"], json!([]), "{app_clean:#}");

            let named_model = cases
                .iter()
                .find(|case| case.7 == "update:title")
                .expect("model event case should exist");
            model_props::assert_define_model_prop_navigation(
                &client,
                &app_uri,
                &app_source,
                &named_model.0,
                &named_model.1,
            )
            .await;

            for (
                uri,
                _source,
                usage,
                usage_end,
                declaration,
                reference,
                reference_end,
                name,
                ty,
                renamed,
                rename_supported,
            ) in &cases
            {
                let clean = pull_diagnostics(&client, uri).await;
                assert_eq!(clean["items"], json!([]), "{clean:#}");
                assert_prop_navigation(&client, &app_uri, usage, name, ty, uri, declaration).await;
                let references = references(&client, &app_uri, usage).await;
                assert_no_generated_uri_or_zero_range(&references);
                assert!(
                    contains_location_range(&references, &app_uri, usage, usage_end),
                    "{references:#}"
                );
                assert!(
                    contains_location_range(&references, uri, reference, reference_end),
                    "{references:#}"
                );
                assert!(
                    !contains_location(&references, uri, &zero),
                    "references must not retain a generated fallback: {references:#}"
                );
                let rename = rename(&client, &app_uri, usage, renamed).await;
                if *rename_supported {
                    assert_no_generated_uri_or_zero_range(&rename);
                    assert!(
                        contains_text_edit(&rename, &app_uri, usage, usage_end, renamed),
                        "{rename:#}"
                    );
                    assert!(
                        contains_text_edit(&rename, uri, reference, reference_end, renamed),
                        "{rename:#}"
                    );
                    assert!(
                        !serde_json::to_string(&rename).unwrap().contains(".vue.ts"),
                        "{rename:#}"
                    );
                } else {
                    assert!(rename.is_null(), "{rename:#}");
                }
            }

            let broken_handler =
                app_source.replace("@select=\"handleSelect\"", "@select=\"handleSubmit\"");
            overlay
                .replace(&app_document_uri, broken_handler.as_str())
                .unwrap();
            let broken = pull_diagnostics(&client, &app_uri).await;
            let broken_text = serde_json::to_string(&broken).unwrap();
            assert!(broken_text.contains("2322"), "{broken:#}");
            let event_start =
                position(&broken_handler, broken_handler.find("@select").unwrap() + 1);
            assert!(
                broken["items"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|diagnostic| { diagnostic["range"]["start"] == event_start }),
                "{broken:#}"
            );

            let partial = app_source
                .replace("@activate", "@act")
                .replace("@choose", "@cho")
                .replace("@confirm", "@con")
                .replace("@submit", "@sub")
                .replace("@cancel", "@can")
                .replace("@select", "@sel")
                .replace("@update:modelValue", "@update:m")
                .replace("@update:title", "@update:t");
            overlay
                .replace(&app_document_uri, partial.as_str())
                .unwrap();
            for (usage, label, payload) in [
                ("@act", "activate", "\\\"slot\\\""),
                ("@cho", "choose", "\\\"top-level\\\""),
                ("@con", "confirm", "\\\"conditional\\\""),
                ("@sub", "submit", "boolean"),
                ("@can", "cancel", "string"),
                ("@sel", "select", "\\\"nested\\\""),
                ("@update:m", "update:modelValue", "number"),
                ("@update:t", "update:title", "string"),
            ] {
                let completion_position =
                    position(&partial, partial.find(usage).unwrap() + usage.len());
                assert_completion(&client, &app_uri, &completion_position, label, &[payload]).await;
            }

            stop.store(true, Ordering::Relaxed);
            client.graceful_close().await.unwrap();
            responder.join().unwrap();
        });
    });
}
