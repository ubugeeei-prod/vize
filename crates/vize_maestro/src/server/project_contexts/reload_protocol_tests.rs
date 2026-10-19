//! Configuration reload dispatch through the real LSP protocol.
use super::*;

async fn protocol_send(
    service: &mut LspService<MaestroServer>,
    value: Value,
) -> FixtureResult<Option<Value>> {
    use tower::Service;
    futures::future::poll_fn(|cx| service.poll_ready(cx)).await?;
    Ok(service
        .call(serde_json::from_value::<tower_lsp::jsonrpc::Request>(
            value,
        )?)
        .await?
        .map(serde_json::to_value)
        .transpose()?)
}

#[test]
fn registered_client_watcher_reloads_imported_vite_policy_through_the_protocol() {
    use futures::{SinkExt, StreamExt};
    block_on(crate::runtime::timeout(std::time::Duration::from_secs(30), async {
        let root = tempfile::tempdir().unwrap();
        file(
            root.path(),
            "vite.config.mjs",
            "export default { vue: {} };",
        )
        .unwrap();
        let helper = file(
            root.path(),
            "shared-policy.mjs",
            "export default { languageServer: { hover: false } };",
        )
        .unwrap();
        for package in ["a", "b"] {
            file(root.path(), &cstr!("packages/{package}/package.json"), "{}").unwrap();
            let config = if package == "a" {
                "import vue from '../../shared-policy.mjs'; export default { vue };"
            } else {
                "export default { vue: {} };"
            };
            file(
                root.path(),
                &cstr!("packages/{package}/vite.config.mjs"),
                config,
            )
            .unwrap();
        }
        let (mut service, mut socket) = LspService::new(|client| {
            let mut server = MaestroServer::new(client);
            server.initial_diagnostics = None;
            server
        });
        let root_uri = Url::from_directory_path(root.path()).unwrap();
        let initialized = protocol_send(&mut service, json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":root_uri,"capabilities":{"workspace":{"didChangeWatchedFiles":{"dynamicRegistration":true}}},"initializationOptions":{"lint":false,"typecheck":false,"ecosystem":false}}})).await.unwrap().unwrap();
        assert!(initialized.get("error").is_none());
        let register = async {
            loop {
                let request = socket.next().await.expect("client registration request");
                if request.method() == "client/registerCapability" {
                    let params = request.params().unwrap().clone();
                    socket
                        .send(tower_lsp::jsonrpc::Response::from_ok(
                            request.id().unwrap().clone(),
                            Value::Null,
                        ))
                        .await
                        .unwrap();
                    break params;
                }
            }
        };
        let (notification, registration) = futures::join!(
            protocol_send(
                &mut service,
                json!({"jsonrpc":"2.0","method":"initialized","params":{}})
            ),
            register
        );
        let scenario = async {
        assert_eq!(notification.unwrap(), None);
        let registration = &registration["registrations"][0];
        assert_eq!(registration["method"], "workspace/didChangeWatchedFiles");
        assert!(registration["registerOptions"]["watchers"].as_array().unwrap().iter().any(|watcher| watcher["globPattern"] == "**/*.{vue,ts,tsx,mts,cts,js,jsx,mjs,cjs}"));
        let source = "<template><div /></template>\n<style>.app { color:red }</style>";
        let a = file(root.path(), "packages/a/src/App.vue", "<template />").unwrap();
        let b = file(root.path(), "packages/b/src/App.vue", "<template />").unwrap();
        for uri in [&a, &b] {
            assert_eq!(protocol_send(&mut service, json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"vue","version":7,"text":source}}})).await.unwrap(), None);
        }
        let hover = |id: i32, uri: &Url| json!({"jsonrpc":"2.0","id":id,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":{"line":1,"character":15}}});
        let before_a = protocol_send(&mut service, hover(2, &a)).await.unwrap().unwrap();
        assert_eq!(before_a, json!({"jsonrpc":"2.0","id":2,"result":null}));
        let before_b = protocol_send(&mut service, hover(3, &b)).await.unwrap().unwrap();
        assert!(before_b["result"].is_object());
        let sibling = service.inner().state.document_project_state(&b).unwrap();
        std::fs::write(
            helper.to_file_path().unwrap(),
            "export default { languageServer: { hover: true } };",
        )
        .unwrap();
        assert_eq!(protocol_send(&mut service, json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":helper,"type":2}]}})).await.unwrap(), None);
        let after_a = protocol_send(&mut service, hover(4, &a)).await.unwrap().unwrap();
        assert_eq!(
            after_a,
            json!({"jsonrpc":"2.0","id":4,"result":before_b["result"]})
        );
        let after_b = protocol_send(&mut service, hover(5, &b)).await.unwrap().unwrap();
        assert_eq!(
            after_b,
            json!({"jsonrpc":"2.0","id":5,"result":before_b["result"]})
        );
        assert!(Arc::ptr_eq(
            &sibling,
            &service.inner().state.document_project_state(&b).unwrap()
        ));
        assert_eq!(
            service.inner().state.documents.text(&a).as_deref(),
            Some(source)
        );
        assert_eq!(service.inner().state.documents.version(&a), Some(7));
        };
        // Keep the client reading while notifications and requests are sent.
        let client = async {
            while let Some(request) = socket.next().await {
                if let Some(id) = request.id() {
                    socket.send(tower_lsp::jsonrpc::Response::from_ok(id.clone(), Value::Null)).await.unwrap();
                }
            }
        };
        futures::pin_mut!(scenario, client);
        if let futures::future::Either::Right(_) = futures::future::select(scenario, client).await {
            panic!("client disconnected before the watcher scenario completed");
        }
    })).expect("registered watcher protocol completes within 30 seconds");
}

#[test]
fn watched_reference_failures_keep_dispatch_and_native_environment_of_each_full_key() {
    use tower_lsp::lsp_types::DidChangeWatchedFilesParams;
    use vize_l0::config::VueVersion;

    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    file(&root, "package.json", "{}").unwrap();
    file(&root, "vite.config.mjs", "export default {};").unwrap();
    file(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.legacy.json"},{"path":"./tsconfig.modern.json"}]}"#,
    )
    .unwrap();
    let leaf_contents = r#"{"extends":"./legacy-base.json","include":["src/legacy/**/*.vue"]}"#;
    let leaf = file(&root, "tsconfig.legacy.json", leaf_contents).unwrap();
    let base = file(
        &root,
        "legacy-base.json",
        r#"{"vueCompilerOptions":{"target":2.7}}"#,
    )
    .unwrap();
    file(
        &root,
        "tsconfig.modern.json",
        r#"{"include":["src/modern/**/*.vue"],"vueCompilerOptions":{"target":3.5}}"#,
    )
    .unwrap();
    let source = "<script lang=\"ts\">export const unsavedReference = 1;</script><template><div /></template>";
    let legacy = file(&root, "src/legacy/App.vue", "<template>disk</template>").unwrap();
    let modern = file(&root, "src/modern/App.vue", "<template>disk</template>").unwrap();
    let (service, socket) = LspService::new(|client| {
        let mut server = MaestroServer::new(client);
        server.initial_diagnostics = None;
        server
    });
    drop(socket);
    let server = service.inner();
    server.state.set_workspace_root(root.clone());
    server.state.set_workspace_folders(vec![root.clone()]);
    server.state.load_workspace_config(&root);
    server.state.apply_lsp_initialization_options(Some(
        &json!({"lint":false,"typecheck":false,"ecosystem":false}),
    ));
    for uri in [&legacy, &modern] {
        server
            .state
            .documents
            .open(uri.clone(), source.into(), 19, "vue".into());
    }
    let mut previous = block_on(server.for_document(&legacy)).state.clone();
    let control = block_on(server.for_document(&modern)).state.clone();
    let control_stamp = control.corsa_request_stamp();
    assert_eq!(previous.type_checker_vue_version(), VueVersion::V2_7);
    assert_eq!(control.type_checker_vue_version(), VueVersion::V3);
    for (changed, invalid, repair, expected) in [
        (
            &base,
            Some(r#"{"vueCompilerOptions":{"target":4}}"#),
            r#"{"vueCompilerOptions":{"target":3.5}}"#,
            VueVersion::V3,
        ),
        // A deleted base would be skipped like `tsc`; a malformed one rejects.
        (
            &base,
            Some("{"),
            r#"{"vueCompilerOptions":{"target":2.7}}"#,
            VueVersion::V2_7,
        ),
        (&leaf, None, leaf_contents, VueVersion::V2_7),
    ] {
        let path = changed.to_file_path().unwrap();
        let before = previous.corsa_request_stamp();
        let checker = previous.get_type_checker_config();
        let dialect = previous.type_checker_vue_version();
        assert!(previous.linter_settings_for_uri(&legacy).is_some());
        let typ = if let Some(invalid) = invalid {
            std::fs::write(&path, invalid).unwrap();
            FileChangeType::CHANGED
        } else {
            std::fs::remove_file(&path).unwrap();
            FileChangeType::DELETED
        };
        block_on(tower_lsp::LanguageServer::did_change_watched_files(
            server,
            DidChangeWatchedFilesParams {
                changes: vec![FileEvent {
                    uri: changed.clone(),
                    typ,
                }],
            },
        ));
        // The shell successfully reloaded, while the selected leaf failed.
        // Query production dispatch after the actual watcher, not the registry.
        let retained = block_on(server.for_document(&legacy));
        assert!(Arc::ptr_eq(&previous, &retained.state));
        assert!(before.is_current(&retained.state));
        assert_eq!(retained.state.get_type_checker_config(), checker);
        assert_eq!(retained.state.type_checker_vue_version(), dialect);
        assert!(retained.state.linter_settings_for_uri(&legacy).is_some());
        assert_eq!(
            retained.state.documents.text(&legacy).as_deref(),
            Some(source)
        );
        assert_eq!(retained.state.documents.version(&legacy), Some(19));
        assert!(
            retained
                .state
                .get_virtual_docs(&legacy)
                .unwrap()
                .all()
                .iter()
                .any(|doc| doc.content.contains("unsavedReference"))
        );
        let unaffected = block_on(server.for_document(&modern));
        assert!(Arc::ptr_eq(&control, &unaffected.state));
        assert!(control_stamp.is_current(&unaffected.state));
        assert_eq!(unaffected.state.type_checker_vue_version(), VueVersion::V3);
        std::fs::write(&path, repair).unwrap();
        block_on(tower_lsp::LanguageServer::did_change_watched_files(
            server,
            DidChangeWatchedFilesParams {
                changes: vec![FileEvent {
                    uri: changed.clone(),
                    typ: if invalid.is_some() {
                        FileChangeType::CHANGED
                    } else {
                        FileChangeType::CREATED
                    },
                }],
            },
        ));
        let repaired = block_on(server.for_document(&legacy));
        assert!(!Arc::ptr_eq(&previous, &repaired.state));
        assert!(!before.is_current(&previous));
        assert_eq!(repaired.state.type_checker_vue_version(), expected);
        assert_eq!(
            repaired.state.documents.text(&legacy).as_deref(),
            Some(source)
        );
        assert_eq!(repaired.state.documents.version(&legacy), Some(19));
        assert!(Arc::ptr_eq(
            &control,
            &block_on(server.for_document(&modern)).state
        ));
        assert!(control_stamp.is_current(&control));
        previous = repaired.state.clone();
    }
}
