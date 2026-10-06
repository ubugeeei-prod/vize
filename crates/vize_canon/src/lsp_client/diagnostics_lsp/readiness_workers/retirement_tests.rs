#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "whole controlled Corsa socket frames and joined native owner laws"
)]

use super::{MAX_IN_FLIGHT, run};
use crate::corsa_bridge::worker::BoundedWorker;
use corsa_lsp::jsonrpc::{JsonRpcConnection, JsonRpcConnectionOptions, RpcHandlerMap, read_frame};
use futures::{FutureExt, executor::block_on};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    io::{BufReader, Write},
    os::unix::net::UnixStream,
    sync::mpsc,
    time::Duration,
};
use vize_l0::cstr;

const SETTLE: Duration = Duration::from_secs(10);
const DOCUMENTS: usize = 534;

fn frame(reader: &mut BufReader<UnixStream>) -> Value {
    serde_json::from_slice(&read_frame(reader).expect("complete request frame"))
        .expect("whole JSON")
}

fn respond(peer: &mut UnixStream, request: &Value, result: Value) {
    let bytes =
        serde_json::to_vec(&json!({"jsonrpc":"2.0", "id":request["id"], "result":result})).unwrap();
    write!(peer, "Content-Length: {}\r\n\r\n", bytes.len()).unwrap();
    peer.write_all(&bytes).unwrap();
    peer.flush().unwrap();
}

fn symbols(uri: &str) -> Value {
    json!([{"name":uri,"kind":13,"range":{"start":{"line":0,"character":0},"end":{"line":2,"character":0}},
        "selectionRange":{"start":{"line":1,"character":6},"end":{"line":1,"character":10}},"children":[]}])
}

fn hover() -> Value {
    serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp-regressions/readiness-after-retirement-7698/hover.json")))
        .expect("complete independently authored hover")
}

struct Owner {
    client: JsonRpcConnection,
    dirty: Vec<std::string::String>,
    accepted: bool,
}

impl Owner {
    fn query(&mut self) -> Result<Value, vize_l0::String> {
        run(&self.dirty, |uri| {
            let result = corsa::runtime::block_on(self.client.request_value(
                "textDocument/documentSymbol",
                json!({"textDocument":{"uri":uri}}),
            ))
            .map_err(|error| {
                let corsa::CorsaError::Rpc(ref payload) = error else {
                    panic!("expected whole controlled native refusal, got {error:?}");
                };
                assert_eq!(serde_json::to_value(payload).unwrap(), refusal());
                cstr!("{error}")
            })?;
            assert_eq!(result, symbols(uri));
            Ok(())
        })?;
        crate::corsa_bridge::native_operation::checkpoint()?;
        self.dirty.clear();
        self.accepted = true;
        corsa::runtime::block_on(self.client.request_value("textDocument/hover", json!({
            "textDocument":{"uri":"file:///workspace/Host000.vue.ts"},"position":{"line":1,"character":15}
        }))).map_err(|error| cstr!("{error}"))
    }
}

fn refusal() -> Value {
    json!({"code":-32603,"message":"Broken pipe","data":{"phase":"readiness","generation":7}})
}

#[test]
fn retired_caller_drains_reversed_window_then_same_owner_rechecks_all_534_and_closes() {
    retired_readiness_session(DOCUMENTS, false);
}

#[test]
fn retired_reversed_window_preserves_completed_native_error_before_live_recovery() {
    retired_readiness_session(DOCUMENTS, true);
}

#[test]
fn retired_single_owner_preserves_completed_native_error_before_live_recovery() {
    retired_readiness_session(1, true);
}

fn retired_readiness_session(documents: usize, fail: bool) {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp-regressions/readiness-after-retirement-7698/Source.ts.txt"
    ));
    assert_eq!(
        &source.lines().nth(1).expect("authored declaration")[13..18],
        "value"
    );
    let (socket, mut peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(SETTLE)).unwrap();
    let client = JsonRpcConnection::try_spawn_with_options(
        BufReader::new(socket.try_clone().unwrap()),
        socket,
        RpcHandlerMap::default(),
        JsonRpcConnectionOptions::new().with_request_timeout(Some(SETTLE)),
    )
    .unwrap();
    let uris = (0..documents)
        .map(|index| format!("file:///workspace/Host{index:03}.vue.ts"))
        .collect::<Vec<_>>();
    let expected = uris.iter().cloned().collect::<BTreeSet<_>>();
    let worker = BoundedWorker::new(
        "vize-retired-readiness-test",
        Owner {
            client,
            dirty: uris,
            accepted: false,
        },
    );
    let (held, entered) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let mut reader = BufReader::new(peer.try_clone().unwrap());
        let requests = (0..documents.min(MAX_IN_FLIGHT))
            .map(|_| frame(&mut reader))
            .collect::<Vec<_>>();
        for request in &requests {
            assert_eq!(
                request,
                &json!({"jsonrpc":"2.0","id":request["id"],
                "method":"textDocument/documentSymbol","params":{"textDocument":{"uri":request["params"]["textDocument"]["uri"]}}})
            );
        }
        held.send(()).unwrap();
        wait.recv_timeout(SETTLE).unwrap();
        for (index, request) in requests.iter().rev().enumerate() {
            let uri = request["params"]["textDocument"]["uri"].as_str().unwrap();
            if fail && index + 1 == requests.len() {
                let bytes = serde_json::to_vec(
                    &json!({"jsonrpc":"2.0","id":request["id"],"error":refusal()}),
                )
                .unwrap();
                write!(peer, "Content-Length: {}\r\n\r\n", bytes.len()).unwrap();
                peer.write_all(&bytes).unwrap();
                peer.flush().unwrap();
            } else {
                respond(&mut peer, request, symbols(uri));
            }
        }
        // The next live caller must re-ACK the complete generation. There is
        // no abandoned-caller semantic request or partial acceptance frame.
        let mut seen = BTreeSet::new();
        for start in (0..documents).step_by(MAX_IN_FLIGHT) {
            let count = (documents - start).min(MAX_IN_FLIGHT);
            let requests = (0..count).map(|_| frame(&mut reader)).collect::<Vec<_>>();
            for request in requests.iter().rev() {
                assert_eq!(request["method"], "textDocument/documentSymbol");
                let uri = request["params"]["textDocument"]["uri"].as_str().unwrap();
                assert!(seen.insert(uri.to_owned()));
                respond(&mut peer, request, symbols(uri));
            }
        }
        assert_eq!(seen, expected);
        let query = frame(&mut reader);
        assert_eq!(
            query,
            json!({"jsonrpc":"2.0","id":query["id"],"method":"textDocument/hover",
            "params":{"textDocument":{"uri":"file:///workspace/Host000.vue.ts"},"position":{"line":1,"character":15}}})
        );
        respond(&mut peer, &query, hover());
        let shutdown = frame(&mut reader);
        assert_eq!(
            shutdown,
            json!({"jsonrpc":"2.0","id":shutdown["id"],"method":"shutdown"})
        );
        respond(&mut peer, &shutdown, Value::Null);
    });
    let (failure, observed) = mpsc::channel();
    let mut first = Box::pin(worker.submit_ready_async(SETTLE, move |owner| {
        let result = owner.query();
        assert_eq!(owner.dirty.len(), documents);
        assert!(!owner.accepted);
        failure.send(result).unwrap();
    }));
    assert_eq!(first.as_mut().now_or_never(), None);
    entered.recv_timeout(SETTLE).unwrap();
    drop(first);
    assert!(worker.is_draining());
    release.send(()).unwrap();
    assert_eq!(
        observed.recv_timeout(SETTLE).unwrap().unwrap_err(),
        if fail {
            "rpc error -32603: Broken pipe"
        } else {
            "Native caller retired before completing its operation"
        }
    );
    assert_eq!(
        block_on(worker.submit_ready_async(SETTLE, |owner| {
            let response = owner.query().unwrap();
            assert!(owner.accepted);
            assert!(owner.dirty.is_empty());
            let shutdown =
                corsa::runtime::block_on(owner.client.request_without_params::<Value>("shutdown"))
                    .unwrap();
            assert_eq!(shutdown, Value::Null);
            corsa::runtime::block_on(owner.client.close()).unwrap();
            response
        })),
        Ok(hover())
    );
    assert!(!worker.is_draining());
    server.join().unwrap();
}
