//! Server-initiated request handling for the editor LSP transport.

use corsa::runtime::BroadcastReceiver;
use corsa_lsp::{LspClient, jsonrpc::InboundEvent};
use serde_json::Value;
use std::{
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, Ordering},
        mpsc::RecvTimeoutError,
    },
    time::{Duration, Instant},
};

/// Bound on [`Inbound::settle`]. An editor's `shutdown` reply waits for this
/// close, and VS Code gives its whole `stop` 2 s. A watch update still queued
/// behind other background work is left to cancel, as before #12526.
pub(super) const WATCH_SETTLE_TIMEOUT: Duration = Duration::from_millis(250);

/// tsgo's background watch work, observed from its log by the responder
/// thread and, while a shutdown settles, by the closing thread.
pub(super) struct Inbound {
    client: LspClient,
    /// A subscription of its own, never held while the responder blocks, so a
    /// closing thread takes it over without waiting out a responder poll.
    logs: Mutex<BroadcastReceiver<InboundEvent>>,
    watches: Mutex<BackgroundWatches>,
}

impl Inbound {
    pub(super) fn new(client: LspClient) -> Self {
        let logs = Mutex::new(client.subscribe());
        Self {
            client,
            logs,
            watches: Mutex::new(BackgroundWatches::default()),
        }
    }

    fn observe(&self, event: &InboundEvent) {
        if let InboundEvent::Notification { method, params } = event
            && method == "window/logMessage"
        {
            self.watches
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .observe(params);
        }
    }

    /// Count every log already received. `Err` means the transport is gone.
    fn drain(&self, logs: &BroadcastReceiver<InboundEvent>) -> Result<(), RecvTimeoutError> {
        loop {
            match logs.recv_timeout(Duration::ZERO) {
                Ok(event) => self.observe(&event),
                Err(RecvTimeoutError::Timeout) => return Ok(()),
                Err(RecvTimeoutError::Disconnected) => return Err(RecvTimeoutError::Disconnected),
            }
        }
    }

    /// Wait, up to `timeout`, until every snapshot tsgo has adopted so far has
    /// finished its watch update. The responder keeps answering the watch
    /// registrations meanwhile.
    ///
    /// Every log that arrived before the caller's last response is already
    /// queued, so draining the queue here counts each snapshot that request
    /// adopted. A session without client-side watching has nothing to wait
    /// for, and an expired wait leaves teardown as it was.
    pub(super) fn settle(&self, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        let logs = self.logs.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            if self.drain(&logs).is_err()
                || self
                    .watches
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .settled()
            {
                return;
            }
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return;
            };
            match logs.recv_timeout(remaining) {
                Ok(event) => self.observe(&event),
                Err(_) => return,
            }
        }
    }
}

/// Answers the server-initiated requests tsgo makes during startup; without a
/// reply the server blocks before it ever serves an editor request.
pub(super) fn spawn_responder(
    inbound: Arc<Inbound>,
    stop: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    let events = inbound.client.subscribe();
    std::thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            match events.recv_timeout(Duration::from_millis(50)) {
                Ok(InboundEvent::Request { id, method, params }) => {
                    let response = match method.as_ref() {
                        "workspace/configuration" => configuration_response(&params),
                        _ => Value::Null,
                    };
                    let _ = inbound.client.respond(id, response);
                }
                Ok(InboundEvent::Notification { .. }) | Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            // Keep the log queue short; a settling shutdown owns it instead.
            if let Ok(logs) = inbound.logs.try_lock()
                && inbound.drain(&logs).is_err()
            {
                break;
            }
        }
    })
}

/// Snapshot work tsgo reports through its log after a request has answered.
///
/// With client-side watching, tsgo adopts each new snapshot and registers its
/// file watches in the background, and `shutdown` does not wait for that work.
/// Ending the transport while it still awaits a watch registration cancels the
/// request, and tsgo then logs `errors updating watches: … context canceled`
/// to stderr because the canceled transport cannot carry the log (#12526).
#[derive(Default)]
pub(super) struct BackgroundWatches {
    client_side: bool,
    adopted: u64,
    settled: u64,
}

impl BackgroundWatches {
    pub(super) fn observe(&mut self, params: &Value) {
        let Some(message) = params.get("message").and_then(Value::as_str) else {
            return;
        };
        if message.starts_with("file watching: using LSP client-side watching") {
            self.client_side = true;
        } else if message.starts_with("Adopted snapshot ") {
            self.adopted = self.adopted.saturating_add(1);
        } else if message.starts_with("Updated watches in ")
            || message.starts_with("errors updating watches: ")
        {
            self.settled = self.settled.saturating_add(1);
        }
    }

    /// Whether no adopted snapshot is still updating its watches.
    pub(super) fn settled(&self) -> bool {
        !self.client_side || self.settled >= self.adopted
    }
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
