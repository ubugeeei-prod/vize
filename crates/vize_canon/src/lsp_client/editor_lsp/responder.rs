//! Server-initiated request handling for the editor LSP transport.

use corsa_lsp::{LspClient, jsonrpc::InboundEvent};
use serde_json::Value;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::RecvTimeoutError,
    },
    time::Duration,
};

/// Answers the server-initiated requests tsgo makes during startup; without a
/// reply the server blocks before it ever serves an editor request.
pub(super) fn spawn_responder(
    client: LspClient,
    stop: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    let events = client.subscribe();
    std::thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            let InboundEvent::Request { id, method, params } =
                (match events.recv_timeout(Duration::from_millis(50)) {
                    Ok(event) => event,
                    Err(RecvTimeoutError::Timeout) => continue,
                    Err(RecvTimeoutError::Disconnected) => break,
                })
            else {
                continue;
            };

            let response = match method.as_ref() {
                "workspace/configuration" => configuration_response(&params),
                _ => Value::Null,
            };
            let _ = client.respond(id, response);
        }
    })
}

/// `workspace/configuration` results are positional: the array must hold one
/// entry per requested item, in request order, with `null` for settings the
/// client cannot supply. We supply none, so every slot is `null`. A bare `[]`
/// would misalign servers that read `result[i]` for `items[i]`.
fn configuration_response(params: &Value) -> Value {
    let requested = params
        .get("items")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    Value::Array(vec![Value::Null; requested])
}
