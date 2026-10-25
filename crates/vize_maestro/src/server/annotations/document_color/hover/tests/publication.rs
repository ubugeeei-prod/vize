//! Existing dispatch publication stamps reject a completed color reply after
//! an async provider yields, including equal-version close/reopen.

use crate::server::{MaestroServer, project_contexts::DocumentServer};
use futures::{FutureExt, channel::oneshot, executor::block_on};
use tower_lsp::{
    LspService,
    jsonrpc::Error,
    lsp_types::{
        HoverParams, Position, TextDocumentContentChangeEvent, TextDocumentIdentifier,
        TextDocumentPositionParams, Url,
    },
};

#[test]
fn completed_whole_color_hover_is_refused_after_async_source_mutation() {
    for mutation in ["unchanged", "edit", "close", "same-version-reopen"] {
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
            let response = owner
                .local_hover_response(&params)
                .unwrap()
                .unwrap()
                .unwrap();
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
                "edit" => {
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
                "close" => {
                    server.state.documents.close(&uri);
                }
                "same-version-reopen" => {
                    server.state.documents.close(&uri);
                    server.state.documents.open(
                        uri.clone(),
                        source.replace("#336699", "#ff0000"),
                        1,
                        "vue".into(),
                    );
                    assert_eq!(server.state.documents.version(&uri), Some(1));
                }
                "unchanged" => {}
                _ => unreachable!(),
            }
            release.send(()).unwrap();
            let actual = pending.await;
            if mutation == "unchanged" {
                assert_eq!(serde_json::to_value(actual.unwrap()).unwrap(), wanted);
            } else {
                assert_eq!(actual, Err(Error::content_modified()), "{mutation}");
            }
            assert!(!server.state.has_corsa_bridge());
        });
    }
}
