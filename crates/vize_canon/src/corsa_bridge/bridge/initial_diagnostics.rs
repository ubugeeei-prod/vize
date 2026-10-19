//! Split an initial read from foreground state and return an inert result.
#![expect(
    clippy::disallowed_types,
    reason = "owned SDK and mirror leases cross workers"
)]

use std::{sync::Arc, time::Duration};

use super::{CorsaBridge, CorsaBridgeError};
use crate::{
    corsa_bridge::{EditorMirrorSession, LspDiagnostic, worker::WorkerError},
    lsp_client::InitialDiagnosticsRead,
};

pub struct InitialApiDiagnostics {
    pub(super) read: Arc<InitialDiagnosticsRead>,
    // Keep physical mirror storage alive until the entered RPC drains. This
    // does not override explicit shutdown/clear or make mutable files immutable.
    pub(super) mirror: Arc<EditorMirrorSession>,
}

pub struct PendingInitialApiDiagnostics {
    pub(super) read: Arc<InitialDiagnosticsRead>,
    pub(super) result: Result<Option<Vec<lsp_types::Diagnostic>>, CorsaBridgeError>,
}

pub enum InitialApiDiagnosticsOutcome {
    Stale,
    Unsupported,
    Complete(Vec<LspDiagnostic>),
    Failed(CorsaBridgeError),
}

impl CorsaBridge {
    /// Only this read-only lane owns the lease during synchronous native IPC.
    /// Cancelling its future leaves an entered request draining on that lane.
    pub async fn read_initial_api_diagnostics(
        &self,
        request: InitialApiDiagnostics,
    ) -> PendingInitialApiDiagnostics {
        let read = Arc::clone(&request.read);
        let result = self
            .initial_diagnostics_worker
            .submit_async(
                Duration::from_millis(self.config.timeout_ms.max(1)),
                move |_| {
                    let _mirror = request.mirror;
                    request
                        .read
                        .run()
                        .map_err(CorsaBridgeError::CommunicationError)
                },
            )
            .await;
        let result = match result {
            Ok(result) => result,
            Err(WorkerError::TimedOut) => Err(CorsaBridgeError::Timeout),
            Err(WorkerError::Stopped) => Err(CorsaBridgeError::ProcessTerminated),
        };
        PendingInitialApiDiagnostics { read, result }
    }
}

#[cfg(all(test, unix))]
mod tests {
    //! Framed API controls; these do not claim native server scheduling fairness.
    #![expect(clippy::disallowed_macros, reason = "framed protocol test fixtures")]

    use std::{
        io::{BufReader, BufWriter},
        os::unix::net::UnixListener,
        sync::{Arc, mpsc},
        thread,
        time::Duration,
    };

    use corsa::{
        api::{ApiClient, ProjectSession},
        jsonrpc::{RawMessage, RequestId, read_frame, write_frame},
        runtime::block_on,
    };
    use futures::FutureExt;
    use serde_json::{Value, json};

    use super::{CorsaBridge, InitialApiDiagnosticsOutcome};
    use crate::lsp_client::CorsaProjectClient;

    const BOUND: Duration = Duration::from_secs(10);
    const CONFIG: &str = "/workspace/tsconfig.json";
    const FILE: &str = "/workspace/App.vue.ts";
    // session_paths materializes the missing virtual document under this overlay.
    const RPC_FILE: &str = "/workspace/node_modules/.vize/corsa-overlay/App.vue.ts";
    const URI: &str = "file:///workspace/App.vue.ts";
    const OLD: &str = "n0000000000000001";
    const NEW: &str = "n0000000000000002";

    struct Peer {
        _directory: tempfile::TempDir,
        client: ApiClient,
        entered: mpsc::Receiver<()>,
        worker: thread::JoinHandle<Vec<std::string::String>>,
    }

    impl Peer {
        fn start(expected_releases: usize) -> Self {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("initial.sock");
            let listener = UnixListener::bind(&path).unwrap();
            let (entered, observed) = mpsc::sync_channel(1);
            let worker = thread::spawn(move || {
                let (stream, _) = listener.accept().unwrap();
                stream.set_read_timeout(Some(BOUND)).unwrap();
                stream.set_write_timeout(Some(BOUND)).unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                let mut generation = 0;
                let mut pending: Option<RequestId> = None;
                let mut released = Vec::new();
                let mut finished = false;
                loop {
                    let body = match read_frame(&mut reader) {
                        Ok(body) => body,
                        Err(corsa::CorsaError::Closed(_)) => return released,
                        Err(error) => panic!("initial API peer: {error}"),
                    };
                    let request: RawMessage = serde_json::from_slice(&body).unwrap();
                    let id = request.id.unwrap();
                    let params = request.params.unwrap_or(Value::Null);
                    let result = match request.method.as_deref().unwrap() {
                        "initialize" => {
                            json!({"useCaseSensitiveFileNames":true,"currentDirectory":"/workspace"})
                        }
                        "updateSnapshot" => {
                            generation += 1;
                            json!({"snapshot":if generation == 1 { OLD } else { NEW },"projects":[project()],"changes":{"changedProjects":{},"removedProjects":[]}})
                        }
                        "getDefaultProjectForFile" => project(),
                        "getDiagnosticsForFile" => {
                            assert_eq!(params["file"], RPC_FILE);
                            assert_eq!(params["project"], "p./workspace/tsconfig.json");
                            assert_eq!(params["snapshot"], OLD);
                            assert!(pending.replace(id).is_none());
                            entered.send(()).unwrap();
                            continue;
                        }
                        "test/finish" => {
                            finished = true;
                            let id = pending.take().unwrap();
                            if params["fail"] == true {
                                let body = serde_json::to_vec(&json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":"old world transport failed"}})).unwrap();
                                write_frame(&mut writer, &body).unwrap();
                            } else {
                                assert!(
                                    !released.iter().any(|handle| handle == OLD),
                                    "snapshot released before held read finished"
                                );
                                respond(
                                    &mut writer,
                                    id,
                                    json!({"file":RPC_FILE,"semantic":[diagnostic()]}),
                                );
                            }
                            Value::Null
                        }
                        "release" => {
                            let handle = params["handle"].as_str().unwrap().to_owned();
                            assert!(!released.contains(&handle), "double release");
                            released.push(handle);
                            Value::Null
                        }
                        method => panic!("unexpected initial API peer method: {method}"),
                    };
                    respond(&mut writer, id, result);
                    if finished && released.len() == expected_releases {
                        return released;
                    }
                }
            });
            let client = block_on(ApiClient::connect_pipe(path)).unwrap();
            Self {
                _directory: directory,
                client,
                entered: observed,
                worker,
            }
        }

        fn bridge(&self) -> Arc<CorsaBridge> {
            let session = block_on(ProjectSession::open(
                self.client.clone(),
                CONFIG,
                Some(FILE.into()),
            ))
            .unwrap();
            let mut client = CorsaProjectClient::empty_for_test("/workspace".into());
            client
                .did_open_fast(URI, "export const value: string = 1")
                .unwrap();
            *client.project_session_for_test() = Some(session);

            let bridge = Arc::new(CorsaBridge::new());
            block_on(bridge.submit_async(move |slot| {
                *slot = Some(client);
                Ok(())
            }))
            .unwrap();
            bridge
                .initialized
                .store(true, std::sync::atomic::Ordering::SeqCst);
            bridge
        }

        fn release_read(&self, fail: bool) {
            block_on(
                self.client
                    .raw_json_request("test/finish", json!({"fail":fail})),
            )
            .unwrap();
        }

        fn finish(self, bridge: Arc<CorsaBridge>) -> Vec<std::string::String> {
            // Cleanup explicitly releases the current snapshot before client
            // close, whose existing semantics invalidate every remaining lease.
            block_on(bridge.with_client(|client| {
                if let Some(session) = client.project_session_for_test().as_ref() {
                    block_on(session.snapshot().release()).unwrap();
                }
                Ok(())
            }))
            .unwrap();
            block_on(bridge.shutdown()).unwrap();
            drop(bridge);
            block_on(self.client.close()).unwrap();
            self.worker.join().unwrap()
        }
    }

    fn project() -> Value {
        json!({"id":"p./workspace/tsconfig.json","configFileName":CONFIG,"compilerOptions":{"strict":true},"rootFiles":[FILE]})
    }

    fn diagnostic() -> Value {
        json!({"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"severity":1,"code":2322,"source":"ts","message":"Type number is not assignable to string"})
    }

    fn respond(writer: &mut impl std::io::Write, id: RequestId, result: Value) {
        write_frame(
            writer,
            &serde_json::to_vec(&RawMessage::response(id, result)).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn current_read_commits_whole_cache_only_after_foreground_lane_remains_usable() {
        let peer = Peer::start(1);
        let bridge = peer.bridge();
        let request = block_on(bridge.prepare_initial_api_diagnostics(URI))
            .unwrap()
            .unwrap();
        let mut read = Box::pin(bridge.read_initial_api_diagnostics(request));
        assert!(read.as_mut().now_or_never().is_none());
        peer.entered.recv_timeout(BOUND).unwrap();
        // The mutable foreground lane remains usable while the native read waits.
        assert_eq!(
            block_on(bridge.with_client(|client| Ok(client.diagnostics_cache_len()))).unwrap(),
            0
        );
        peer.release_read(false);
        let pending = block_on(read);
        let prior = {
            let mut diagnostic = diagnostic();
            diagnostic["message"] = json!("prior complete cached result");
            diagnostic
        };
        let previous = prior.clone();
        block_on(bridge.with_client(move |client| {
            client.replace_initial_diagnostics_cache(
                URI,
                Some(vec![serde_json::from_value(previous).unwrap()]),
            );
            Ok(())
        }))
        .unwrap();
        assert!(matches!(
            block_on(bridge.finish_initial_api_diagnostics(pending, |commit| {
                assert!(commit());
                false
            }))
            .unwrap(),
            InitialApiDiagnosticsOutcome::Stale
        ));
        assert_eq!(
            serde_json::to_value(
                block_on(bridge.with_client(|client| Ok(client.get_diagnostics(URI)))).unwrap()
            )
            .unwrap(),
            json!([prior])
        );
        let request = block_on(bridge.prepare_initial_api_diagnostics(URI))
            .unwrap()
            .unwrap();
        let mut read = Box::pin(bridge.read_initial_api_diagnostics(request));
        assert!(read.as_mut().now_or_never().is_none());
        peer.entered.recv_timeout(BOUND).unwrap();
        peer.release_read(false);
        let pending = block_on(read);
        let outcome =
            block_on(bridge.finish_initial_api_diagnostics(pending, |commit| commit())).unwrap();
        let InitialApiDiagnosticsOutcome::Complete(diagnostics) = outcome else {
            panic!("expected whole diagnostic result")
        };
        let mut expected = diagnostic();
        expected["relatedInformation"] = Value::Null;
        assert_eq!(
            serde_json::to_value(diagnostics).unwrap(),
            json!([expected])
        );
        assert_eq!(
            serde_json::to_value(
                block_on(bridge.with_client(|client| Ok(client.get_diagnostics(URI)))).unwrap()
            )
            .unwrap(),
            json!([diagnostic()])
        );
        assert_eq!(peer.finish(bridge), vec![OLD]);
    }

    #[test]
    fn refreshed_snapshot_rejects_both_old_success_and_old_error_without_effects() {
        for fail in [false, true] {
            let peer = Peer::start(2);
            let bridge = peer.bridge();
            let request = block_on(bridge.prepare_initial_api_diagnostics(URI))
                .unwrap()
                .unwrap();
            let mut read = Box::pin(bridge.read_initial_api_diagnostics(request));
            assert!(read.as_mut().now_or_never().is_none());
            peer.entered.recv_timeout(BOUND).unwrap();
            block_on(bridge.with_client(|client| {
                block_on(
                    client
                        .project_session_for_test()
                        .as_mut()
                        .unwrap()
                        .refresh(None),
                )
                .unwrap();
                assert_eq!(
                    client
                        .project_session_for_test()
                        .as_ref()
                        .unwrap()
                        .snapshot()
                        .handle
                        .as_str(),
                    NEW
                );
                Ok(())
            }))
            .unwrap();
            peer.release_read(fail);
            let pending = block_on(read);
            assert!(matches!(
                block_on(bridge.finish_initial_api_diagnostics(pending, |commit| commit()))
                    .unwrap(),
                InitialApiDiagnosticsOutcome::Stale
            ));
            assert!(bridge.is_initialized());
            assert_eq!(
                block_on(bridge.with_client(|client| Ok(client.diagnostics_cache_len()))).unwrap(),
                0
            );
            let mut released = peer.finish(bridge);
            released.sort();
            assert_eq!(released, vec![OLD, NEW]);
        }
    }

    #[test]
    fn cancelled_caller_keeps_real_snapshot_alive_until_entered_read_drains() {
        let peer = Peer::start(2);
        let bridge = peer.bridge();
        let request = block_on(bridge.prepare_initial_api_diagnostics(URI))
            .unwrap()
            .unwrap();
        let weak = block_on(bridge.with_client(|client| {
            Ok(Arc::downgrade(
                &client
                    .project_session_for_test()
                    .as_ref()
                    .unwrap()
                    .snapshot_lease(),
            ))
        }))
        .unwrap();
        let mut read = Box::pin(bridge.read_initial_api_diagnostics(request));
        assert!(read.as_mut().now_or_never().is_none());
        peer.entered.recv_timeout(BOUND).unwrap();
        drop(read);
        assert!(bridge.is_draining());
        block_on(bridge.with_client(|client| {
            block_on(
                client
                    .project_session_for_test()
                    .as_mut()
                    .unwrap()
                    .refresh(None),
            )
            .unwrap();
            Ok(())
        }))
        .unwrap();
        assert!(
            weak.upgrade().is_some(),
            "entered read owns the retired snapshot"
        );
        peer.release_read(false);
        block_on(
            bridge
                .initial_diagnostics_worker
                .submit_ready_async(BOUND, |_| ()),
        )
        .unwrap();
        assert!(
            weak.upgrade().is_none(),
            "last worker lease releases after actual RPC completion"
        );
        assert!(!bridge.is_draining());
        assert_eq!(
            block_on(bridge.with_client(|client| Ok(client.diagnostics_cache_len()))).unwrap(),
            0
        );
        let mut released = peer.finish(bridge);
        released.sort();
        assert_eq!(released, vec![OLD, NEW]);
    }

    #[test]
    fn dropped_session_owner_does_not_drop_a_workers_snapshot_or_admit_its_result() {
        let peer = Peer::start(1);
        let bridge = peer.bridge();
        let request = block_on(bridge.prepare_initial_api_diagnostics(URI))
            .unwrap()
            .unwrap();
        let mut read = Box::pin(bridge.read_initial_api_diagnostics(request));
        assert!(read.as_mut().now_or_never().is_none());
        peer.entered.recv_timeout(BOUND).unwrap();
        block_on(bridge.with_client(|client| {
            drop(client.project_session_for_test().take());
            Ok(())
        }))
        .unwrap();
        peer.release_read(false);
        let pending = block_on(read);
        assert!(matches!(
            block_on(bridge.finish_initial_api_diagnostics(pending, |commit| commit())).unwrap(),
            InitialApiDiagnosticsOutcome::Stale
        ));
        assert_eq!(peer.finish(bridge), vec![OLD]);
    }

    #[test]
    fn explicit_release_still_invalidates_the_remote_snapshot_without_double_release() {
        let peer = Peer::start(1);
        let bridge = peer.bridge();
        let request = block_on(bridge.prepare_initial_api_diagnostics(URI))
            .unwrap()
            .unwrap();
        let mut read = Box::pin(bridge.read_initial_api_diagnostics(request));
        assert!(read.as_mut().now_or_never().is_none());
        peer.entered.recv_timeout(BOUND).unwrap();
        block_on(bridge.with_client(|client| {
            block_on(
                client
                    .project_session_for_test()
                    .as_ref()
                    .unwrap()
                    .snapshot()
                    .release(),
            )
            .unwrap();
            Ok(())
        }))
        .unwrap();
        peer.release_read(true);
        let pending = block_on(read);
        assert!(matches!(
            block_on(bridge.finish_initial_api_diagnostics(pending, |commit| commit())).unwrap(),
            InitialApiDiagnosticsOutcome::Failed(_)
        ));
        assert!(
            bridge.is_initialized(),
            "adapter cannot retire a session on its own"
        );
        assert_eq!(peer.finish(bridge), vec![OLD]);
    }
}
