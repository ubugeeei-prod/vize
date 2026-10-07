use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use corsa_lsp::{LspClient, LspSpawnConfig, VirtualDocument, jsonrpc::InboundEvent};
use lsp_types::Uri;
use serde_json::json;
use vize_l0::cstr;

pub mod content_mapper_lsp_support;
use content_mapper_lsp_support::raw_requests::{
    RawInitialize, RawInitialized, RawSetContentMapperContributions,
};
use content_mapper_lsp_support::{
    EditorResponder, StopOnDrop, assert_hover, copy_fixture, editor_capabilities, file_uri, hover,
    install_packages, pull_diagnostics, workspace_root,
};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";

#[test]
fn standard_tsgo_lsp_accepts_authored_vue_content_mapper_contribution() {
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
    let app_path = project.path().join("src/App.vue");
    let app_source = std::fs::read_to_string(&app_path).unwrap();
    let app_uri = file_uri(&app_path);

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
            // Keep this exact-upstream LSP lane focused on mapper wiring.
            // The pinned native-preview server accepts the Content Mapper
            // contribution here, while authored Vue diagnostics and symbol
            // behavior are covered by Vize's editor/LSP oracles plus the exact
            // tsgo CLI conformance tests.

            let clean = pull_diagnostics(&client, &child_uri).await;
            assert_eq!(clean["items"], json!([]), "{clean:#}");
            assert_script_hover_during_template_recovery(&client, project.path()).await;
            stop.store(true, Ordering::Relaxed);
            client.graceful_close().await.unwrap();
            responder.join().unwrap();
        });
    });
}

async fn assert_script_hover_during_template_recovery(client: &LspClient, project: &Path) {
    let corpus = workspace_root()
        .join("tests/_fixtures/differential/typechecker/content-mapper-incomplete-tag");
    let oracle: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(corpus.join("expectations.json")).unwrap())
            .unwrap();
    let path = project.join("src/Recovery.vue");
    let uri_text = file_uri(&path);
    let uri = Uri::from_str(&uri_text).unwrap();
    let overlay = client.overlay();
    for case in oracle["cases"].as_array().unwrap() {
        let original =
            std::fs::read_to_string(corpus.join(case["file"].as_str().unwrap())).unwrap();
        for newline in ["\n", "\r\n"] {
            let complete = original.replace('\n', newline);
            std::fs::write(&path, &complete).unwrap();
            overlay
                .open(VirtualDocument::new(uri.clone(), "vue", complete.as_str()))
                .unwrap();
            let template = complete.find("<template>").unwrap();
            let unfinished = cstr!(
                "{}<template><Loc</template>{newline}",
                complete.get(..template).unwrap()
            );
            let repaired = complete.replace("'broken'", "1");
            let start = json!({
                "line": case["line"].as_u64().unwrap() - 1,
                "character": case["column"].as_u64().unwrap() - 1
            });
            let end = json!({
                "line": case["line"].as_u64().unwrap() - 1,
                "character": case["column"].as_u64().unwrap() + 3
            });
            for (state, source) in [
                ("complete", complete.as_str()),
                ("unfinished-tag", unfinished.as_str()),
                ("template-repaired", complete.as_str()),
                ("script-repaired", repaired.as_str()),
            ] {
                overlay.replace(&uri, source).unwrap();
                let actual = hover(client, &uri_text, &start).await;
                assert_hover(&actual, &start, &end, oracle["scriptHover"].clone(), state);
            }
            overlay.close(&uri).unwrap();
        }
    }
}
