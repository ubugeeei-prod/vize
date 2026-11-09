//! Component props, emits, slots and their JSDoc in hover through tsgo's own
//! LSP with the Vize content mapper (the editor backend without `vize lsp`).
#![expect(clippy::disallowed_methods, reason = "fixture URIs are std strings")]

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use corsa_lsp::{LspClient, LspSpawnConfig, VirtualDocument, jsonrpc::InboundEvent};
use lsp_types::Uri;
use serde_json::{Map, Value, json};

use crate::content_mapper_lsp_support;
use crate::lsp_component_api_hover_cli::{CASES, FILES};
use content_mapper_lsp_support::raw_requests::{
    RawInitialize, RawInitialized, RawSetContentMapperContributions,
};
use content_mapper_lsp_support::{
    EditorResponder, StopOnDrop, editor_capabilities, file_uri, hover, install_packages, position,
    pull_diagnostics, workspace_root,
};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";

#[test]
fn standard_tsgo_lsp_hovers_component_api_with_jsdoc() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping Content Mapper component API hover: {TSGO_ENV} is not set");
        return;
    };
    let expected: Value = serde_json::from_str(include_str!(
        "fixtures/component_api_hover/hover.tsgo-mapper.json"
    ))
    .unwrap();
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-lsp-hover-")
        .tempdir_in(cases_root)
        .unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/component_api_hover");
    std::fs::copy(
        fixture.join("tsconfig.json"),
        project.path().join("tsconfig.json"),
    )
    .unwrap();
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    let documents: Vec<_> = FILES
        .iter()
        .map(|(name, source)| {
            let path = project.path().join("src").join(name);
            std::fs::write(&path, source).unwrap();
            (*name, *source, file_uri(&path).to_string())
        })
        .collect();
    install_packages(project.path());
    let root_uri = file_uri(project.path());

    let stop = AtomicBool::new(false);
    let editor = EditorResponder::default();
    let mut actual = Map::new();
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
            client
                .request::<RawInitialize>(json!({
                    "processId": std::process::id(),
                    "rootUri": root_uri,
                    "workspaceFolders": [{ "uri": root_uri, "name": "content-mapper-hover" }],
                    "capabilities": editor_capabilities(),
                    "initializationOptions": { "runExternalCode": true }
                }))
                .await
                .unwrap();
            client.notify::<RawInitialized>(json!({})).unwrap();
            let open_documents: Vec<_> = documents
                .iter()
                .filter(|(name, _, _)| name.ends_with(".vue"))
                .map(|(_, _, uri)| json!({ "uri": uri }))
                .collect();
            client
                .request::<RawSetContentMapperContributions>(json!({
                    "contributions": [{
                        "contributorId": "vize",
                        "extensions": [".vue"],
                        "inferredProjectContribution": {
                            "options": {},
                            "manifest": {
                                "name": "vize",
                                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                                "compilerOptions": ["noUnusedLocals"],
                                "dynamicConfig": true
                            }
                        }
                    }],
                    "openDocuments": open_documents
                }))
                .await
                .unwrap();
            let overlay = client.overlay();
            for (name, source, uri) in &documents {
                let language = if name.ends_with(".vue") {
                    "vue"
                } else {
                    "typescript"
                };
                overlay
                    .open(VirtualDocument::new(
                        Uri::from_str(uri).unwrap(),
                        language,
                        *source,
                    ))
                    .unwrap();
            }
            // A pull loads the project and every mapped SFC before the first hover.
            for (name, _, uri) in &documents {
                let diagnostics = pull_diagnostics(&client, uri).await;
                assert_eq!(diagnostics["items"], json!([]), "{name}: {diagnostics:#}");
            }
            for (case, file, needle) in CASES {
                let (_, source, uri) = documents.iter().find(|(name, _, _)| *name == file).unwrap();
                let offset = source.find(needle).expect("fixture needle");
                let response = hover(&client, uri, &position(source, offset)).await;
                actual.insert(case.into(), response);
            }
            stop.store(true, Ordering::Relaxed);
            client.graceful_close().await.unwrap();
            responder.join().unwrap();
        });
    });
    let actual = Value::Object(actual);
    if actual != expected {
        // The whole packet, so a provider update can be reviewed and pinned at once.
        eprintln!("{}", serde_json::to_string_pretty(&actual).unwrap());
    }
    for (case, _, _) in CASES {
        assert_eq!(actual[case], expected[case], "{case}");
    }
}
