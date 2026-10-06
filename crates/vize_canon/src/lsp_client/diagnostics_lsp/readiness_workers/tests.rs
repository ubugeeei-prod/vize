#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "transport laws retain complete JSON frames and Unix socket peers"
)]

use super::{MAX_IN_FLIGHT, run};
use corsa::runtime::block_on;
use corsa_lsp::jsonrpc::{JsonRpcConnection, JsonRpcConnectionOptions, RpcHandlerMap, read_frame};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    io::{BufReader, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};
use vize_l0::cstr;

const DOCUMENTS: usize = 534;

fn connection() -> (JsonRpcConnection, UnixStream, UnixStream) {
    let (socket, peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let control = socket.try_clone().unwrap();
    let client = JsonRpcConnection::try_spawn_with_options(
        BufReader::new(socket.try_clone().unwrap()),
        socket,
        RpcHandlerMap::default(),
        JsonRpcConnectionOptions::new().with_request_timeout(Some(Duration::from_secs(5))),
    )
    .unwrap();
    (client, peer, control)
}

fn request(reader: &mut BufReader<UnixStream>) -> Value {
    let frame = read_frame(reader).unwrap();
    let request: Value = serde_json::from_slice(&frame).unwrap();
    assert_eq!(request["method"], "textDocument/documentSymbol");
    request
}

fn full_result(uri: &str) -> Value {
    json!([{
        "name": uri,
        "kind": 13,
        "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 2, "character": 0 } },
        "selectionRange": { "start": { "line": 1, "character": 6 }, "end": { "line": 1, "character": 10 } },
        "children": []
    }])
}

fn respond(peer: &mut UnixStream, response: Value) {
    let bytes = serde_json::to_vec(&response).unwrap();
    write!(peer, "Content-Length: {}\r\n\r\n", bytes.len()).unwrap();
    peer.write_all(&bytes).unwrap();
    peer.flush().unwrap();
}

struct Active<'a>(&'a AtomicUsize);

impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[test]
fn all_534_full_responses_overlap_with_bound_16_and_match_their_uris() {
    let (client, mut peer, _control) = connection();
    let server = thread::spawn(move || {
        let mut reader = BufReader::new(peer.try_clone().unwrap());
        let mut seen = BTreeSet::new();
        for start in (0..DOCUMENTS).step_by(MAX_IN_FLIGHT) {
            let count = (DOCUMENTS - start).min(MAX_IN_FLIGHT);
            // No response is sent until this complete window arrives. A
            // single-thread blocking future cannot submit even the second URI.
            let requests = (0..count).map(|_| request(&mut reader)).collect::<Vec<_>>();
            for request in requests.into_iter().rev() {
                let uri = request["params"]["textDocument"]["uri"].as_str().unwrap();
                assert!(seen.insert(uri.to_owned()));
                respond(
                    &mut peer,
                    json!({
                        "jsonrpc": "2.0", "id": request["id"], "result": full_result(uri)
                    }),
                );
            }
        }
        seen
    });
    let uris = (0..DOCUMENTS)
        .map(|index| format!("file:///workspace/Host{index:03}.vue.ts"))
        .collect::<Vec<_>>();
    let active = AtomicUsize::new(0);
    let peak = AtomicUsize::new(0);
    let complete = AtomicUsize::new(0);
    run(&uris, |uri| {
        let current = active.fetch_add(1, Ordering::SeqCst) + 1;
        peak.fetch_max(current, Ordering::SeqCst);
        let _active = Active(&active);
        let result = block_on(client.request_value(
            "textDocument/documentSymbol",
            json!({ "textDocument": { "uri": uri } }),
        ))
        .map_err(|error| cstr!("{error}"))?;
        assert_eq!(result, full_result(uri));
        complete.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert_eq!(active.load(Ordering::SeqCst), 0);
    assert_eq!(peak.load(Ordering::SeqCst), MAX_IN_FLIGHT);
    assert_eq!(complete.load(Ordering::SeqCst), DOCUMENTS);
    let expected = uris.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(server.join().unwrap(), expected);
    block_on(client.close()).unwrap();
}

#[test]
fn native_error_is_returned_only_after_every_started_request_drains() {
    let (client, mut peer, control) = connection();
    let started = Arc::new(AtomicUsize::new(0));
    let responses = Arc::new(Mutex::new(Vec::new()));
    let captured = responses.clone();
    let server = thread::spawn(move || {
        let mut reader = BufReader::new(peer.try_clone().unwrap());
        let requests = (0..MAX_IN_FLIGHT)
            .map(|_| request(&mut reader))
            .collect::<Vec<_>>();
        let mut requests = requests.into_iter();
        let first = requests.next().unwrap();
        respond(
            &mut peer,
            json!({
                "jsonrpc": "2.0", "id": first["id"],
                "error": { "code": -32603, "message": "whole readiness refusal" }
            }),
        );
        for request in requests {
            let uri = request["params"]["textDocument"]["uri"].as_str().unwrap();
            respond(
                &mut peer,
                json!({
                    "jsonrpc": "2.0", "id": request["id"], "result": full_result(uri)
                }),
            );
        }
        // Other already-entered workers may have submitted their next URI
        // before the refusal was observed. Those requests must also drain.
        while let Ok(frame) = read_frame(&mut reader) {
            let request: Value = serde_json::from_slice(&frame).unwrap();
            let uri = request["params"]["textDocument"]["uri"].as_str().unwrap();
            respond(
                &mut peer,
                json!({
                    "jsonrpc": "2.0", "id": request["id"], "result": full_result(uri)
                }),
            );
        }
    });
    let active = AtomicUsize::new(0);
    let uris = (0..DOCUMENTS)
        .map(|index| format!("file:///workspace/Host{index:03}.vue.ts"))
        .collect::<Vec<_>>();
    let result = run(&uris, |uri| {
        started.fetch_add(1, Ordering::SeqCst);
        active.fetch_add(1, Ordering::SeqCst);
        let _active = Active(&active);
        let result = block_on(client.request_value(
            "textDocument/documentSymbol",
            json!({ "textDocument": { "uri": uri } }),
        ));
        captured
            .lock()
            .unwrap()
            .push((uri.clone(), format!("{result:?}")));
        result.map(|_| ()).map_err(|error| cstr!("{error}"))
    });
    assert!(result.unwrap_err().contains("whole readiness refusal"));
    assert_eq!(active.load(Ordering::SeqCst), 0);
    assert_eq!(
        responses.lock().unwrap().len(),
        started.load(Ordering::SeqCst)
    );
    control.shutdown(Shutdown::Both).unwrap();
    server.join().unwrap();
    block_on(client.close()).unwrap();
}

#[test]
fn empty_generation_starts_no_request() {
    run::<u8>(&[], |_| unreachable!("no URI to acknowledge")).unwrap();
}
