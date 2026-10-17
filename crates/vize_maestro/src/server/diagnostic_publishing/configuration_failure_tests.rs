//! A failed authored compiler project is terminal feedback, not backend death.

use futures::{FutureExt, StreamExt, channel::mpsc};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};
use tower::Service;
use tower_lsp::{LspService, lsp_types::Url};

use super::MaestroServer;

async fn send(
    service: &mut LspService<MaestroServer>,
    packet: Value,
) -> Result<Option<Value>, Box<dyn std::error::Error>> {
    futures::future::poll_fn(|cx| service.poll_ready(cx)).await?;
    service
        .call(serde_json::from_value(packet)?)
        .await?
        .map(serde_json::to_value)
        .transpose()
        .map_err(Into::into)
}

async fn publication(
    messages: &mut mpsc::UnboundedReceiver<Value>,
    uri: &Url,
) -> Result<Value, Box<dyn std::error::Error>> {
    loop {
        let packet = messages.next().await.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "client transport closed")
        })?;
        if packet.get("method") == Some(&json!("textDocument/publishDiagnostics"))
            && packet.get("params").and_then(|params| params.get("uri")) == Some(&json!(uri))
        {
            return Ok(packet);
        }
    }
}

#[test]
fn missing_compiler_base_publishes_once_keeps_backend_and_repairs_after_watch() {
    // Stay under the real workspace runtime resolution boundary. No fake
    // checker or extra interactive request is needed to trigger diagnostics.
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let cases = workspace.join("target/vize-tests/configuration-failure");
    std::fs::create_dir_all(&cases).unwrap();
    let directory = tempfile::tempdir_in(cases).unwrap();
    let root = std::fs::canonicalize(directory.path()).unwrap();
    let config = root.join("tsconfig.json");
    let base = root.join("base.json");
    let host = root.join("App.vue");
    const SOURCE: &str = "<script setup lang=\"ts\">\nconst value: string = null;\n</script>\n<template>{{ value }}</template>\n";
    std::fs::write(&host, SOURCE).unwrap();
    std::fs::write(&config, r#"{"extends":"./base.json","include":["*.vue"]}"#).unwrap();
    let uri = Url::from_file_path(&host).unwrap();
    let (mut service, mut socket) = LspService::new(MaestroServer::new);
    let (messages, mut received) = mpsc::unbounded();
    let (stop, stopped) = futures::channel::oneshot::channel();

    let packets = crate::runtime::block_on(async {
        let client = async move {
            let mut packets = Vec::new();
            let stopped = stopped.fuse();
            futures::pin_mut!(stopped);
            loop {
                futures::select! {
                    _ = stopped => break,
                    packet = socket.next().fuse() => {
                        let packet = packet.unwrap();
                        assert!(packet.id().is_none(), "no dynamic watcher capability was offered");
                        let packet = serde_json::to_value(packet).unwrap();
                        messages.unbounded_send(packet.clone()).unwrap();
                        packets.push(packet);
                    }
                }
            }
            while let Some(Some(packet)) = socket.next().now_or_never() {
                packets.push(serde_json::to_value(packet).unwrap());
            }
            packets
        };
        let scenario = async {
            let initialized = send(
                &mut service,
                json!({
                    "jsonrpc":"2.0","id":1,"method":"initialize","params":{
                        "rootUri":Url::from_directory_path(&root).unwrap(),"capabilities":{},
                        "initializationOptions":{"lint":false,"typecheck":true,"ecosystem":false}
                    }
                }),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(initialized.get("id"), Some(&json!(1)));
            assert!(initialized.get("error").is_none());
            assert!(
                initialized
                    .get("result")
                    .and_then(|r| r.get("capabilities"))
                    .is_some_and(Value::is_object)
            );
            assert_eq!(
                send(
                    &mut service,
                    json!({"jsonrpc":"2.0","method":"initialized","params":{}})
                )
                .await
                .unwrap(),
                None
            );
            assert_eq!(
                send(
                    &mut service,
                    json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                        "textDocument":{"uri":uri,"languageId":"vue","version":1,"text":SOURCE}
                    }})
                )
                .await
                .unwrap(),
                None
            );
            let failure = publication(&mut received, &uri).await.unwrap();
            assert_eq!(
                failure,
                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{
                    "uri":uri,"version":1,"diagnostics":[{
                        "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},
                        "severity":1,"code":"configuration-error","source":"vize/types",
                        "message":vize_l0::cstr!("Configuration error: cannot resolve tsconfig extends \"./base.json\" from {}",config.display())
                    }]
                }})
            );
            let project = service.inner().for_document_admitted(&uri).await;
            let bridge = project.state.get_corsa_bridge().await.unwrap();
            let stamp = project.state.corsa_request_stamp();
            assert!(stamp.is_current(&project.state));
            let repeated = crate::ide::DiagnosticService::collect_async(&project.state, &uri).await;
            assert_eq!(
                Some(&serde_json::to_value(repeated).unwrap()),
                failure
                    .get("params")
                    .and_then(|params| params.get("diagnostics"))
            );
            assert!(
                stamp.is_current(&project.state),
                "configuration failure must not advance the native epoch"
            );
            let retained = project.state.get_corsa_bridge().await.unwrap();
            assert!(
                std::ptr::eq(bridge.as_ref(), retained.as_ref()),
                "the actual backend must survive a local configuration error"
            );
            assert!(bridge.is_initialized());
            drop(project);

            std::fs::write(&base, r#"{"compilerOptions":{"strict":true,"types":[]}}"#).unwrap();
            assert_eq!(
                send(
                    &mut service,
                    json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{
                        "changes":[{"uri":Url::from_file_path(&base).unwrap(),"type":1}]
                    }})
                )
                .await
                .unwrap(),
                None
            );
            let repaired = publication(&mut received, &uri).await.unwrap();
            assert_eq!(
                repaired,
                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{
                    "uri":uri,"version":1,"diagnostics":[{
                        "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":11}},
                        "severity":1,"code":2322,"source":"vize/types",
                        "message":"Type 'null' is not assignable to type 'string'."
                    }]
                }})
            );
            assert!(
                service
                    .inner()
                    .for_document_admitted(&uri)
                    .await
                    .state
                    .has_corsa_bridge()
            );
            assert_eq!(std::fs::read_to_string(&host).unwrap(), SOURCE);
            assert_eq!(
                send(
                    &mut service,
                    json!({"jsonrpc":"2.0","id":2,"method":"shutdown"})
                )
                .await
                .unwrap(),
                Some(json!({"jsonrpc":"2.0","id":2,"result":null}))
            );
            stop.send(()).unwrap();
        };
        let bounded = crate::runtime::timeout(Duration::from_secs(30), scenario);
        let (result, packets) = futures::join!(bounded, client);
        result.unwrap();
        packets
    });
    let diagnostics = packets
        .iter()
        .filter(|packet| packet.get("method") == Some(&json!("textDocument/publishDiagnostics")))
        .collect::<Vec<_>>();
    assert_eq!(
        diagnostics.len(),
        2,
        "one failure and one physical repair, without retry publication"
    );
}
