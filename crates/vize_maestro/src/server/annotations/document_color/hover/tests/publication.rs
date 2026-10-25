//! Existing dispatch publication stamps reject a completed color reply after
//! an async provider yields, including equal-version close/reopen.

use crate::server::{MaestroServer, project_contexts::DocumentServer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutation {
    Unchanged,
    Edit,
    Close,
    SameVersionReopen,
}
use futures::{FutureExt, channel::oneshot, executor::block_on};
use tower_lsp::{
    LanguageServer, LspService,
    jsonrpc::Error,
    lsp_types::{
        HoverParams, Position, TextDocumentContentChangeEvent, TextDocumentIdentifier,
        TextDocumentPositionParams, Url,
    },
};

#[test]
fn completed_whole_color_hover_is_refused_after_async_source_mutation() {
    for mutation in [
        Mutation::Unchanged,
        Mutation::Edit,
        Mutation::Close,
        Mutation::SameVersionReopen,
    ] {
        block_on(async {
            let (service, _socket) = LspService::new(MaestroServer::new);
            let server = service.inner();
            server.state.apply_lsp_initialization_options(Some(
                &serde_json::json!({"editor":true,"hover":true,"typecheck":false,"lint":false}),
            ));
            let uri = Url::parse("file:///css-literal-hover/App.vue").unwrap();
            let source = "<style>/*日本🧭*/ .x { color:#336699; }</style>";
            server
                .state
                .documents
                .open(uri.clone(), source.into(), 1, "vue".into());
            let offset = source.find("#336699").unwrap();
            let (line, character) = crate::ide::offset_to_position(source, offset);
            let params = HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    position: Position::new(line, character),
                },
                work_done_progress_params: Default::default(),
            };
            let owner = DocumentServer::primary(server);
            let response = server.hover(params).await.unwrap().unwrap();
            let wanted = serde_json::json!({"contents":{"kind":"markdown","value":super::DOCUMENTATION},"range":{
                "start":super::position(source,offset),"end":super::position(source,offset+7)
            }});
            assert_eq!(serde_json::to_value(&response).unwrap(), wanted);
            let (release, ready) = oneshot::channel();
            let pending = owner.native_request(async {
                ready.await.unwrap();
                Ok(response)
            });
            futures::pin_mut!(pending);
            assert!(pending.as_mut().now_or_never().is_none());
            match mutation {
                Mutation::Edit => {
                    assert!(server.state.documents.apply_changes(
                        &uri,
                        vec![TextDocumentContentChangeEvent {
                            range: None,
                            range_length: None,
                            text: source.replace("#336699", "#ff0000")
                        }],
                        2
                    ));
                }
                Mutation::Close => {
                    server.state.documents.close(&uri);
                }
                Mutation::SameVersionReopen => {
                    server.state.documents.close(&uri);
                    server.state.documents.open(
                        uri.clone(),
                        source.replace("#336699", "#ff0000"),
                        1,
                        "vue".into(),
                    );
                    assert_eq!(server.state.documents.version(&uri), Some(1));
                }
                Mutation::Unchanged => {}
            }
            release.send(()).unwrap();
            let actual = pending.await;
            if mutation == Mutation::Unchanged {
                assert_eq!(serde_json::to_value(actual.unwrap()).unwrap(), wanted);
            } else {
                assert_eq!(actual, Err(Error::content_modified()), "{mutation:?}");
            }
            assert!(!server.state.has_corsa_bridge());
        });
    }
}

#[test]
fn concrete_color_hover_keeps_the_local_route_with_typechecking_enabled() {
    block_on(async {
        let project = tempfile::tempdir().unwrap();
        let missing = project
            .path()
            .join("definitely-missing-corsa-css-literal-control");
        assert!(!missing.exists());
        let config = serde_json::json!({"vue": {
            "typeChecker": { "corsaPath": missing },
            "languageServer": { "hover": true, "typecheck": true, "lint": false }
        }});
        std::fs::write(
            project.path().join("vite.config.mjs"),
            vize_l0::cstr!("export default {};", config).as_bytes(),
        )
        .unwrap();
        let (service, _socket) = LspService::new(MaestroServer::new);
        let server = service.inner();
        server
            .initialize(tower_lsp::lsp_types::InitializeParams {
                workspace_folders: Some(vec![tower_lsp::lsp_types::WorkspaceFolder {
                    uri: Url::from_directory_path(project.path()).unwrap(),
                    name: "CSS literal route".into(),
                }]),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(server.state.is_lsp_typecheck_enabled());
        assert!(server.state.lsp_features().hover);
        assert_eq!(
            server.state.get_type_checker_config().runtime_path(),
            missing.to_str()
        );
        assert!(!server.state.has_corsa_bridge());
        assert_eq!(server.state.corsa_init_failure(), None);
        let source = "<template><div /></template>\n<style>.日本🧭 { color:#336699; }</style>";
        let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
        server
            .state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        let start = source.find("#336699").unwrap();
        let (line, character) = crate::ide::offset_to_position(source, start + 2);
        let actual = server
            .hover(HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri },
                    position: Position::new(line, character),
                },
                work_done_progress_params: Default::default(),
            })
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::json!({
                "contents": { "kind": "markdown", "value": super::DOCUMENTATION },
                "range": { "start": super::position(source, start), "end": super::position(source, start + 7) }
            })
        );
        assert!(!server.state.has_corsa_bridge());
        assert_eq!(server.state.corsa_init_failure(), None);
        // A real startup attempt against the pinned missing executable proves
        // the preceding hover did not quietly run another discovered provider.
        assert!(server.state.get_corsa_bridge().await.is_none());
        assert!(server.state.corsa_init_failure().is_some());
    });
}
