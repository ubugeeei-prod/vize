//! Opt-in UI constructor metadata. No requests, payload dumps or startup-time hashes.
#![expect(
    clippy::disallowed_types,
    reason = "Corsa's existing observer/profiler contracts require owned Arc callbacks"
)]

use corsa::{
    CorsaEvent, CorsaObserver, SharedObserver,
    api::{ApiMode, ApiProfileEvent, ApiProfiler, ApiSpawnConfig, SharedProfiler},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{OpenOptions, symlink_metadata},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use vize_l0::{String, cstr};

const LIMIT: usize = 256;
static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Caller {
    schema: String,
    caller_attempt_id: String,
    worker_pid: u32,
    worker_index: u32,
    worker_total: u32,
    input_ordinal: u32,
    input_path: String,
    input_sha256: String,
}

struct State {
    start: Instant,
    events: Mutex<Vec<Value>>,
    dropped: AtomicUsize,
    closed: AtomicBool,
}

impl State {
    fn push(&self, event: Value) {
        if self.closed.load(Ordering::Acquire) {
            return;
        }
        if let Ok(mut events) = self.events.try_lock()
            && events.len() < LIMIT
        {
            if self.closed.load(Ordering::Acquire) {
                return;
            }
            events.push(json!({
                "received_elapsed_ns": cstr!("{}", self.start.elapsed().as_nanos()),
                "event": event,
            }));
            return;
        }
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }
}

struct ObserverHook {
    state: Arc<State>,
    previous: Option<SharedObserver>,
}

struct ProfilerHook {
    state: Arc<State>,
    previous: Option<SharedProfiler>,
}

fn method(value: &str) -> &'static str {
    match value {
        "initialize" => "initialize",
        "createSnapshot" => "createSnapshot",
        "updateSnapshot" => "updateSnapshot",
        "parseConfigFile" => "parseConfigFile",
        "describeCapabilities" => "describeCapabilities",
        _ => "other",
    }
}

fn error_shape(value: &str) -> Value {
    let size = match value.len() {
        0 => "empty",
        1..=64 => "1-64-bytes",
        65..=256 => "65-256-bytes",
        257..=4096 => "257-4096-bytes",
        _ => "over-4096-bytes",
    };
    let prefix_ascii = value.as_bytes().iter().take(256).all(u8::is_ascii);
    json!({"shape": size, "prefix_ascii": prefix_ascii, "examined_bytes": value.len().min(256), "text_retained": false})
}

impl CorsaObserver for ObserverHook {
    fn on_event(&self, event: &CorsaEvent) {
        if let Some(previous) = &self.previous {
            previous.on_event(event);
        }
        if self.state.closed.load(Ordering::Acquire) {
            return;
        }
        let value = match event {
            CorsaEvent::JsonRpcRequestTimedOut {
                method: name,
                timeout,
            } => json!({
                "variant": "JsonRpcRequestTimedOut", "method": method(name.as_str()),
                "configured_timeout_ns": cstr!("{}", timeout.as_nanos()),
                "actual_elapsed_wait_proven": false,
            }),
            CorsaEvent::JsonRpcPendingRequestsFailed { error, count } => json!({
                "variant": "JsonRpcPendingRequestsFailed", "count": cstr!("{count}"),
                "normalized_error": error_shape(error.as_str()),
            }),
            CorsaEvent::JsonRpcOutboundQueueFull => json!({"variant": "JsonRpcOutboundQueueFull"}),
            CorsaEvent::JsonRpcLocalHandlerPanicked { method: name } => json!({
                "variant": "JsonRpcLocalHandlerPanicked", "method": method(name.as_str()),
            }),
            CorsaEvent::MsgpackRequestTimedOut {
                method: name,
                timeout,
            } => json!({
                "variant": "MsgpackRequestTimedOut", "method": method(name.as_str()),
                "configured_timeout_ns": cstr!("{}", timeout.as_nanos()),
            }),
            CorsaEvent::MsgpackWorkerQueueFull { method: name } => json!({
                "variant": "MsgpackWorkerQueueFull", "method": method(name.as_str()),
            }),
            CorsaEvent::MsgpackWorkerTerminated { reason } => json!({
                "variant": "MsgpackWorkerTerminated", "normalized_reason": error_shape(reason.as_str()),
            }),
            _ => json!({"variant": "other", "payload_retained": false}),
        };
        self.state.push(value);
    }
}

impl ApiProfiler for ProfilerHook {
    fn on_profile(&self, event: &ApiProfileEvent) {
        if let Some(previous) = &self.previous {
            previous.on_profile(event);
        }
        if self.state.closed.load(Ordering::Acquire) {
            return;
        }
        let transport = match event.transport.as_str() {
            "jsonrpc" => "jsonrpc",
            "msgpack" => "msgpack",
            _ => "other",
        };
        self.state.push(json!({
            "variant": "completed_profile_phase", "method": method(event.method.as_str()),
            "transport": transport, "phase": event.phase.as_str(),
            "duration_ns": cstr!("{}", event.duration.as_nanos()),
            "wire_request_id": null, "transport_samples_are_success_only": true,
        }));
    }
}

pub(super) struct Capture {
    root: PathBuf,
    caller: Caller,
    ordinal: u64,
    utc_ns: String,
    config: Value,
    state: Arc<State>,
}

fn context(filename: &str) -> Option<(PathBuf, Caller)> {
    let configured = std::env::var_os("VIZE_UI_STARTUP_METADATA_DIR")?;
    if std::env::var("GITHUB_ACTIONS").ok().as_deref() != Some("true") {
        return None;
    }
    let selected = PathBuf::from(configured);
    let metadata = symlink_metadata(&selected).ok()?;
    if !selected.is_absolute() || !metadata.is_dir() || metadata.file_type().is_symlink() {
        return None;
    }
    let root = selected.canonicalize().ok()?;
    let temporary = PathBuf::from(std::env::var_os("RUNNER_TEMP")?)
        .canonicalize()
        .ok()?;
    if root == temporary || !root.starts_with(temporary) {
        return None;
    }
    let raw = std::env::var("VIZE_UI_STARTUP_CONTEXT").ok()?;
    if raw.len() > 8192 {
        return None;
    }
    let caller: Caller = serde_json::from_str(&raw).ok()?;
    let expected = cstr!("caller-{}-{}", std::process::id(), caller.input_ordinal);
    if caller.schema != "vize.ui.startup.caller.v1"
        || caller.worker_pid != std::process::id()
        || caller.worker_total == 0
        || caller.worker_total > 8
        || caller.worker_index >= caller.worker_total
        || caller.caller_attempt_id != expected
        || caller.input_path != filename
        || caller.input_path.len() > 4096
        || caller.input_sha256.len() != 64
        || !caller
            .input_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    Some((root, caller))
}

pub(super) fn attach(
    mut config: ApiSpawnConfig,
    filename: &str,
) -> (ApiSpawnConfig, Option<Capture>) {
    let Some((root, caller)) = context(filename) else {
        return (config, None);
    };
    let state = Arc::new(State {
        start: Instant::now(),
        events: Mutex::new(Vec::with_capacity(LIMIT)),
        dropped: AtomicUsize::new(0),
        closed: AtomicBool::new(false),
    });
    let settings = json!({
        "executable": config.command.executable().to_string_lossy(),
        "cwd": config.command.cwd().to_string_lossy(),
        "mode": match config.mode { ApiMode::AsyncJsonRpcStdio => "jsonrpc", ApiMode::SyncMsgpackStdio => "msgpack" },
        "request_timeout_ns": config.request_timeout.map(|value| cstr!("{}", value.as_nanos())),
        "outbound_capacity": config.outbound_capacity,
        "release_queue_capacity": config.release_queue_capacity,
        "run_external_code": config.run_external_code,
        "profiler_was_configured": config.profiler.is_some(),
        "profile_collection": "wrap_existing_sink_only",
    });
    wrap_existing_sinks(&mut config, &state);
    let capture = Capture {
        root,
        caller,
        ordinal: NEXT.fetch_add(1, Ordering::Relaxed),
        utc_ns: cstr!(
            "{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ),
        config: settings,
        state,
    };
    (config, Some(capture))
}

fn wrap_existing_sinks(config: &mut ApiSpawnConfig, state: &Arc<State>) {
    config.observer = Some(Arc::new(ObserverHook {
        state: state.clone(),
        previous: config.observer.take(),
    }));
    if let Some(previous) = config.profiler.take() {
        config.profiler = Some(Arc::new(ProfilerHook {
            state: state.clone(),
            previous: Some(previous),
        }));
    }
}

impl Capture {
    pub(super) fn finish(self, returned: bool) {
        let elapsed = self.state.start.elapsed();
        self.state.closed.store(true, Ordering::Release);
        let closed_elapsed = self.state.start.elapsed();
        let events = self
            .state
            .events
            .lock()
            .map(|mut values| std::mem::take(&mut *values))
            .ok();
        let record = json!({
            "schema": "vize.ui.startup.constructor.v1", "caller": self.caller,
            "constructor_attempt_id": cstr!("constructor-{}-{}", std::process::id(), self.ordinal),
            "identity_kind": "caller_local_not_sdk_wire_id", "sdk_wire_request_id": null,
            "backend_pid": null, "backend_exit_status": null,
            "clock": "rust.Instant-relative", "utc_anchor_ns": self.utc_ns,
            "elapsed_ns": cstr!("{}", elapsed.as_nanos()), "returned": returned,
            "observer_closed_elapsed_ns": cstr!("{}", closed_elapsed.as_nanos()),
            "config": self.config, "events": events,
            "dropped": self.state.dropped.load(Ordering::Relaxed),
            "capture_window": "observer_close_after_original_constructor_outcome",
            "later_cleanup_events_collected": false,
        });
        let name = cstr!("sdk-{}-{}.json", std::process::id(), self.ordinal);
        let written = serde_json::to_vec(&record).ok().and_then(|bytes| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.root.join(name.as_str()))
                .ok()?;
            file.write_all(&bytes).ok()
        });
        if written.is_none() {
            let _ = writeln!(
                std::io::stderr().lock(),
                "Vize UI startup metadata unavailable (redacted)"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use corsa::api::ApiProfilePhase;
    use std::time::Duration;

    fn state() -> Arc<State> {
        Arc::new(State {
            start: Instant::now(),
            events: Mutex::new(Vec::new()),
            dropped: AtomicUsize::new(0),
            closed: AtomicBool::new(false),
        })
    }

    struct Counter(Arc<AtomicUsize>);
    impl CorsaObserver for Counter {
        fn on_event(&self, _: &CorsaEvent) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    impl ApiProfiler for Counter {
        fn on_profile(&self, _: &ApiProfileEvent) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn absent_profiler_stays_absent_and_existing_sinks_survive_close() {
        let state = state();
        let mut absent = ApiSpawnConfig::new("never-executed");
        wrap_existing_sinks(&mut absent, &state);
        assert!(absent.profiler.is_none());
        let observed = Arc::new(AtomicUsize::new(0));
        let profiled = Arc::new(AtomicUsize::new(0));
        let mut config = ApiSpawnConfig::new("never-executed");
        config.observer = Some(Arc::new(Counter(observed.clone())));
        config.profiler = Some(Arc::new(Counter(profiled.clone())));
        wrap_existing_sinks(&mut config, &state);
        let profile = ApiProfileEvent {
            method: "updateSnapshot".into(),
            transport: "jsonrpc".into(),
            phase: ApiProfilePhase::SerializeParams,
            duration: Duration::from_millis(1),
        };
        for closed in [false, true] {
            state.closed.store(closed, Ordering::Release);
            config
                .observer
                .as_ref()
                .unwrap()
                .on_event(&CorsaEvent::JsonRpcOutboundQueueFull);
            config.profiler.as_ref().unwrap().on_profile(&profile);
        }
        assert_eq!(observed.load(Ordering::Relaxed), 2);
        assert_eq!(profiled.load(Ordering::Relaxed), 2);
        assert_eq!(state.events.lock().unwrap().len(), 2);
    }

    #[test]
    fn unknown_methods_and_pending_error_payloads_are_never_retained() {
        let state = state();
        let hook = ObserverHook {
            state: state.clone(),
            previous: None,
        };
        let secret = "sensitive-source-and-request-payload";
        hook.on_event(&CorsaEvent::JsonRpcPendingRequestsFailed {
            error: secret.into(),
            count: 3,
        });
        hook.on_event(&CorsaEvent::JsonRpcLocalHandlerPanicked {
            method: secret.into(),
        });
        let events = state.events.lock().unwrap();
        let raw = serde_json::to_vec(&*events).unwrap();
        assert!(
            !raw.windows(secret.len())
                .any(|part| part == secret.as_bytes())
        );
        assert_eq!(
            events[0]["event"]["variant"],
            "JsonRpcPendingRequestsFailed"
        );
        assert_eq!(events[0]["event"]["count"], "3");
        assert_eq!(
            events[0]["event"]["normalized_error"]["text_retained"],
            false
        );
        assert_eq!(events[1]["event"]["method"], "other");
    }
}
