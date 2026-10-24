//! Real Maestro document requests while its client withholds a watch ACK.

use crate::{runtime::block_on, server::MaestroServer};
use futures::{FutureExt, SinkExt, StreamExt};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use tower::Service;
use tower_lsp::{
    LanguageServer, LspService,
    jsonrpc::{Request, Response},
    lsp_types::Url,
};

pub(super) async fn protocol_send<S: LanguageServer>(
    service: &mut LspService<S>,
    value: Value,
) -> Option<Value> {
    futures::future::poll_fn(|cx| service.poll_ready(cx))
        .await
        .unwrap();
    service
        .call(serde_json::from_value(value).unwrap())
        .await
        .unwrap()
        .map(|response| serde_json::to_value(response).unwrap())
}

#[test]
fn held_external_watch_ack_does_not_delay_real_document_hover() {
    block_on(crate::runtime::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("package.json"), "{}").unwrap();
        let source = "<template><div /></template>\n<style>.app { color:red }</style>";
        let path = root.path().join("App.vue");
        std::fs::write(&path, source).unwrap();
        let uri = Url::from_file_path(path).unwrap();
        let mut expected = None;
        for supported in [false, true] {
            let (mut service, mut socket) = LspService::new(|client| {
                let mut server = MaestroServer::new(client);
                server.initial_diagnostics = None;
                server
            });
            let (observed, held_request) = futures::channel::oneshot::channel();
            let (release, released) = futures::channel::oneshot::channel();
            let (acknowledged, ack) = futures::channel::oneshot::channel();
            let (diagnostic, received_diagnostic) = futures::channel::oneshot::channel();
            // Drain the real bounded ClientSocket throughout the operation.
            // Only the configuration registration ACK is intentionally held.
            let client = async move {
                let mut observed = Some(observed);
                let mut diagnostic = Some(diagnostic);
                let mut held: Option<Request> = None;
                let mut packets = Vec::new();
                let released = released.fuse();
                futures::pin_mut!(released);
                loop {
                    futures::select! {
                        _ = released => {
                            assert_eq!(held.is_some(), supported);
                            if let Some(request) = held {
                                socket
                                    .send(Response::from_ok(
                                        request.id().unwrap().clone(),
                                        Value::Null,
                                    ))
                                    .await
                                    .unwrap();
                            }
                            acknowledged.send(()).unwrap();
                            return packets;
                        },
                        request = socket.next().fuse() => {
                            let request = request.unwrap();
                            eprintln!("real held-watch client packet: {request:?}");
                            if request.method() == "client/registerCapability" {
                                let id = &request.params().unwrap()["registrations"][0]["id"];
                                if id.as_str().unwrap().starts_with("vize-config-watch-")
                                    && observed.is_some()
                                {
                                    assert!(supported);
                                    held = Some(request);
                                    observed.take().unwrap().send(()).unwrap();
                                } else {
                                    assert!(
                                        id == "vize-typecheck-dependencies"
                                            || id.as_str().unwrap().starts_with("vize-config-watch-")
                                    );
                                    socket
                                        .send(Response::from_ok(
                                            request.id().unwrap().clone(),
                                            Value::Null,
                                        ))
                                        .await
                                        .unwrap();
                                }
                            } else {
                                assert!(request.id().is_none(), "unexpected client request");
                                if request.method() == "textDocument/publishDiagnostics"
                                    && let Some(diagnostic) = diagnostic.take()
                                {
                                    diagnostic.send(()).unwrap();
                                }
                                packets.push(request);
                            }
                        }
                    }
                }
            };
            let server = async {
                let response = protocol_send(
                    &mut service,
                    json!({
                        "jsonrpc":"2.0","id":1,"method":"initialize","params":{
                            "rootUri":Url::from_directory_path(root.path()).unwrap(),
                            "capabilities":{"workspace":{"didChangeWatchedFiles":{
                                "dynamicRegistration":supported,"relativePatternSupport":supported
                            }}},
                            "initializationOptions":{"lint":false,"typecheck":false,"ecosystem":false}
                        }
                    }),
                )
                .await
                .unwrap();
                assert!(response.get("error").is_none());
                assert_eq!(
                    protocol_send(
                        &mut service,
                        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
                    )
                    .await,
                    None
                );
                if supported {
                    held_request.await.unwrap();
                }
                assert_eq!(
                    protocol_send(
                        &mut service,
                        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                            "textDocument":{"uri":uri,"languageId":"vue","version":1,"text":source}
                        }})
                    )
                    .await,
                    None
                );
                let started = Instant::now();
                let hover = protocol_send(
                    &mut service,
                    json!({"jsonrpc":"2.0","id":20,"method":"textDocument/hover","params":{
                        "textDocument":{"uri":uri},"position":{"line":1,"character":15}
                    }}),
                )
                .await
                .unwrap();
                assert!(
                    started.elapsed() < Duration::from_millis(100),
                    "local hover must not await watch registration"
                );
                assert!(hover["result"].is_object());
                if let Some(expected) = &expected {
                    assert_eq!(
                        &hover, expected,
                        "the entire hover response survives a held watch ACK"
                    );
                } else {
                    expected = Some(hover);
                }
                assert!(!service.inner().state.has_corsa_bridge());
                assert!(service.inner().state.corsa_init_failure().is_none());
                received_diagnostic.await.unwrap();
                release.send(()).unwrap();
                ack.await.unwrap();
                protocol_send(
                    &mut service,
                    json!({"jsonrpc":"2.0","id":30,"method":"shutdown","params":null}),
                )
                .await;
            };
            let (_, packets) = futures::join!(server, client);
            assert!(
                packets
                    .iter()
                    .any(|packet| packet.method() == "window/logMessage")
            );
            assert!(
                packets
                    .iter()
                    .any(|packet| packet.method() == "textDocument/publishDiagnostics")
            );
        }
    }))
    .expect("watch ACK independence completes with the original local request budget");
}
