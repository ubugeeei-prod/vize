//! Actual initialize packets and ClientSocket registration/ACK lifecycle.

use super::*;
use crate::runtime::block_on;
use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use tower_lsp::lsp_types::{InitializeParams, InitializeResult, InitializedParams};
use tower_lsp::{
    ClientSocket, LanguageServer, LspService,
    jsonrpc::{Request, Response},
};

#[expect(
    clippy::disallowed_types,
    reason = "the fixture needs Weak observers to prove the real root owner is released"
)]
type RootOwner = std::sync::Arc<Control>;

#[cfg(test)]
#[path = "config_watchers_protocol_tests.rs"]
mod protocol;
use protocol::protocol_send;

#[cfg(test)]
#[path = "config_watchers_lifecycle_tests.rs"]
mod lifecycle;

struct Host {
    client: Client,
    control: RootOwner,
}

#[tower_lsp::async_trait]
impl LanguageServer for Host {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        self.control.record_capabilities(&params.capabilities);
        Ok(InitializeResult::default())
    }
    async fn initialized(&self, _: InitializedParams) {
        self.control.start(self.client.clone());
    }
    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        self.control.shutdown();
        Ok(())
    }
}

struct Harness {
    service: LspService<Host>,
    socket: ClientSocket,
    control: RootOwner,
    // Rust drops fields in declaration order: the real socket goes first.
    _transport: Option<TransportLease>,
}

impl Harness {
    fn new(watched: Value) -> Self {
        Self::with_transport(watched, true)
    }
    fn with_transport(watched: Value, acquire_lease: bool) -> Self {
        let control = RootOwner::new(Control::default());
        let root = RootOwner::clone(&control);
        let transport = if acquire_lease {
            control.transport_lease()
        } else {
            None
        };
        let (service, socket) = LspService::new(move |client| Host {
            client,
            control: root,
        });
        let mut host = Self {
            service,
            socket,
            control,
            _transport: transport,
        };
        let response = host
            .send(
                json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
                    "capabilities":{"workspace":{"didChangeWatchedFiles":watched}}
                }}),
            )
            .unwrap();
        assert!(response.get("result").is_some());
        assert!(
            host.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}))
                .is_none()
        );
        host
    }
    fn supported() -> Self {
        Self::new(json!({"dynamicRegistration":true,"relativePatternSupport":true}))
    }
    fn send(&mut self, value: Value) -> Option<Value> {
        block_on(protocol_send(&mut self.service, value))
    }
    fn next(&mut self) -> Request {
        crate::runtime::block_on(crate::runtime::timeout(
            Duration::from_secs(2),
            self.socket.next(),
        ))
        .expect("watch worker must make bounded progress")
        .expect("real client socket remains open")
    }
    fn reply(&mut self, request: &Request, success: bool) {
        let id = request.id().unwrap().clone();
        let response = if success {
            Response::from_ok(id, Value::Null)
        } else {
            Response::from_error(id, tower_lsp::jsonrpc::Error::internal_error())
        };
        crate::runtime::block_on(self.socket.send(response)).unwrap();
    }
    fn idle(&self) {
        until(|| self.control.shared.queue.lock().latest.is_none());
    }
}

fn until(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !predicate() {
        assert!(
            Instant::now() < deadline,
            "worker failed to settle before deadline"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn registration(request: &Request, paths: &[PathBuf]) -> Value {
    assert_eq!(request.method(), "client/registerCapability");
    let params = request.params().unwrap();
    let id = params["registrations"][0]["id"].clone();
    assert!(id.as_str().unwrap().starts_with("vize-config-watch-"));
    let expected: Vec<Value> = paths
        .iter()
        .map(|path| {
            json!({
                "globPattern":{"baseUri":Url::from_directory_path(path.parent().unwrap()).unwrap(),
                    "pattern":path.file_name().unwrap().to_str().unwrap()},"kind":7
            })
        })
        .collect();
    assert_eq!(
        params,
        &json!({"registrations":[{"id":id,"method":METHOD,"registerOptions":{"watchers":expected}}]})
    );
    id
}

fn retired(request: &Request, id: &Value) {
    assert_eq!(request.method(), "client/unregisterCapability");
    assert_eq!(
        request.params(),
        Some(&json!({"unregisterations":[{"id":id,"method":METHOD}]}))
    );
}

#[test]
fn real_client_coalesces_deduplicates_and_retires_only_after_replacement_ack() {
    let mut host = Harness::supported();
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("a.json");
    let b = root.path().join("b.json");
    let c = root.path().join("c.json");
    host.control.update(vec![b.clone(), a.clone(), b.clone()]);
    let first = host.next();
    let first_id = registration(&first, &[a.clone(), b.clone()]);
    host.control.update(vec![a.clone()]);
    host.control.update(vec![c.clone(), c.clone()]);
    host.reply(&first, true);
    let replacement = host.next();
    let replacement_id = registration(&replacement, std::slice::from_ref(&c));
    assert_ne!(first_id, replacement_id);
    host.reply(&replacement, true);
    let retire = host.next();
    retired(&retire, &first_id);
    host.reply(&retire, true);
    host.control.update(vec![c.clone(), c]);
    host.idle();
    host.control.update(vec![]);
    let retire = host.next();
    retired(&retire, &replacement_id);
    host.reply(&retire, true);
    host.control.shutdown();
    until(|| SharedQueue::strong_count(&host.control.shared) == 1);
}

#[test]
fn failed_registration_keeps_active_and_failed_retirement_retries_on_next_update() {
    let mut host = Harness::supported();
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("a.json");
    let b = root.path().join("b.json");
    let c = root.path().join("c.json");
    let d = root.path().join("d.json");
    host.control.update(vec![a.clone()]);
    let first = host.next();
    let first_id = registration(&first, std::slice::from_ref(&a));
    host.reply(&first, true);
    host.control.update(vec![b.clone()]);
    let failed = host.next();
    let failed_id = registration(&failed, &[b]);
    host.reply(&failed, false);
    host.control.update(vec![c.clone()]);
    let replacement = host.next();
    let replacement_id = registration(&replacement, std::slice::from_ref(&c));
    assert_ne!(replacement_id, failed_id);
    host.reply(&replacement, true);
    let retire = host.next();
    retired(&retire, &first_id);
    host.reply(&retire, false);
    assert!(
        crate::runtime::block_on(crate::runtime::timeout(
            Duration::from_millis(100),
            host.socket.next()
        ))
        .is_err(),
        "failed retirement must not spin or re-register"
    );
    host.control.update(vec![d.clone()]);
    let retry = host.next();
    retired(&retry, &first_id);
    host.reply(&retry, true);
    let latest = host.next();
    registration(&latest, &[d]);
    host.reply(&latest, true);
    let retire = host.next();
    retired(&retire, &replacement_id);
    host.reply(&retire, true);
}

#[test]
fn shutdown_cancels_held_ack_releases_worker_and_does_not_capture_root_control() {
    for explicit in [true, false] {
        let mut host = Harness::supported();
        let root = tempfile::tempdir().unwrap();
        host.control.update(vec![root.path().join("a.json")]);
        let first = host.next();
        host.reply(&first, true);
        host.control.update(vec![root.path().join("b.json")]);
        let held = host.next();
        assert_eq!(held.method(), "client/registerCapability");
        host.control.update(vec![root.path().join("latest.json")]);
        if explicit {
            host.control.shutdown();
            until(|| SharedQueue::strong_count(&host.control.shared) == 1);
            assert!(host.control.shared.queue.lock().latest.is_none());
        }
        let root_owner = RootOwner::downgrade(&host.control);
        let queue = SharedQueue::downgrade(&host.control.shared);
        drop(host);
        assert!(
            root_owner.upgrade().is_none(),
            "worker must never retain Control or server state"
        );
        until(|| queue.upgrade().is_none());
    }
}

#[test]
fn incomplete_or_unsupported_capabilities_never_start_worker_or_send_rpc() {
    for watched in [
        json!({}),
        json!({"dynamicRegistration":false,"relativePatternSupport":true}),
        json!({"dynamicRegistration":true}),
        json!({"dynamicRegistration":true,"relativePatternSupport":false}),
    ] {
        let mut host = Harness::new(watched);
        host.control
            .update(vec![std::env::temp_dir().join("policy.json")]);
        assert!(!host.control.shared.queue.lock().started);
        assert_eq!(SharedQueue::strong_count(&host.control.shared), 1);
        assert!(host.socket.next().now_or_never().is_none());
    }
}

#[test]
fn whole_relative_pattern_packet_escapes_literal_glob_filename_and_encoded_parent() {
    let mut host = Harness::supported();
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("project #[] 中文");
    let path = parent.join("\t policy[]{}?* space 日本.json \u{00a0}\u{feff}");
    host.control.update(vec![path]);
    let request = host.next();
    assert_eq!(request.method(), "client/registerCapability");
    let id = request.params().unwrap()["registrations"][0]["id"].clone();
    let base = Url::from_directory_path(&parent).unwrap();
    assert!(
        base.as_str()
            .contains("project%20%23[]%20%E4%B8%AD%E6%96%87/")
    );
    assert_eq!(base.to_file_path().unwrap(), parent);
    assert_eq!(
        request.params(),
        Some(&json!({"registrations":[{"id":id,"method":METHOD,
        "registerOptions":{"watchers":[{"globPattern":{"baseUri":base,
            "pattern":"[\t][ ]policy[[][]][{][}][?][*][ ]space[ ]日本.json[ ][\u{00a0}][\u{feff}]"},"kind":7}]}}]}))
    );
    host.reply(&request, true);
}

#[cfg(windows)]
#[test]
fn windows_drive_and_unc_parents_use_native_url_conversion() {
    for path in [
        PathBuf::from(r"C:\settings dir\vue.json"),
        PathBuf::from(r"\\server\share\settings dir\vue.json"),
    ] {
        let (represented, watches) = watchers(std::slice::from_ref(&path));
        assert_eq!(represented, [path.clone()]);
        let expected = Url::from_directory_path(path.parent().unwrap()).unwrap();
        assert_eq!(
            watches[0].glob_pattern,
            GlobPattern::Relative(RelativePattern {
                base_uri: OneOf::Right(expected),
                pattern: "vue.json".into()
            })
        );
    }
}

#[cfg(unix)]
#[test]
fn unrepresentable_root_and_non_utf8_targets_preserve_valid_watches_without_churn() {
    use std::os::unix::ffi::OsStringExt;

    let mut host = Harness::supported();
    let root = tempfile::tempdir().unwrap();
    let first_path = root.path().join("a.json");
    let next_path = root.path().join("b.json");
    let filesystem_root = PathBuf::from("/");
    let non_utf8 = root
        .path()
        .join(std::ffi::OsString::from_vec(vec![b'p', 0xff]));
    host.control
        .update(vec![filesystem_root.clone(), non_utf8, first_path.clone()]);
    let first = host.next();
    let first_id = registration(&first, std::slice::from_ref(&first_path));
    host.reply(&first, true);
    // Only invalid observations change: the valid ACKed subset is unchanged.
    host.control.update(vec![filesystem_root, first_path]);
    host.idle();
    host.control.update(vec![next_path.clone()]);
    let replacement = host.next();
    let replacement_id = registration(&replacement, &[next_path]);
    assert_ne!(first_id, replacement_id);
    host.reply(&replacement, true);
    let retire = host.next();
    retired(&retire, &first_id);
    host.reply(&retire, true);
}

#[test]
fn every_ecmascript_trim_character_is_a_literal_at_both_filename_edges() {
    let root = tempfile::tempdir().unwrap();
    for character in "\t\n\u{b}\u{c}\r \u{a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}".chars() {
        let filename = cstr!("{character}policy.json{character}");
        let watcher = watcher(&root.path().join(filename.as_str())).unwrap();
        let GlobPattern::Relative(relative) = watcher.glob_pattern else { panic!("exact relative pattern") };
        assert_eq!(relative.pattern, cstr!("[{character}]policy.json[{character}]"));
    }
    for character in ['\u{85}', '\u{200b}', 'a'] {
        assert!(
            !glob_trim_character(character),
            "do not confuse Rust Unicode whitespace with ECMAScript trim"
        );
    }
}
