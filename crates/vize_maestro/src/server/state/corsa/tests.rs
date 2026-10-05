//! Retain the actual startup lane across cancellation, without a retry storm.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "full owned IPC process fixture"
)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::FutureExt;
use serde_json::json;

use super::super::ServerState;

struct BackendCleanup<'a>(&'a Path);
impl BackendCleanup<'_> {
    fn stop(&self) {
        if let Ok(pids) = std::fs::read_to_string(self.0.join("backend.pids")) {
            let _ = std::process::Command::new("kill")
                .args(pids.lines())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }
}
impl Drop for BackendCleanup<'_> {
    fn drop(&mut self) {
        self.stop();
    }
}

fn await_condition(mut ready: impl FnMut() -> bool) {
    let started = Instant::now();
    while !ready() {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "held IPC fixture did not settle"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn held_backend(timeout_ms: u64) -> (tempfile::TempDir, ServerState) {
    let root = tempfile::tempdir().unwrap();
    let backend = root.path().join("held-corsa");
    std::fs::write(
        &backend,
        "#!/bin/sh\necho $$ >> backend.pids\ncat > /dev/null\n",
    )
    .unwrap();
    std::fs::set_permissions(&backend, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"Bundler"}}"#,
    )
    .unwrap();
    std::fs::write(
        root.path().join("vize.config.json"),
        json!({
            "lsp":{"typecheck":true,"lint":false},
            "typeChecker":{"corsaPath":backend,"lspRequestTimeoutMs":timeout_ms}
        })
        .to_string(),
    )
    .unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_owned());
    state.load_lsp_config(root.path());
    (root, state)
}

#[test]
fn cancelled_startup_keeps_one_worker_until_drain_then_reuses_its_current_owner() {
    let (root, state) = held_backend(60_000);
    let cleanup = BackendCleanup(root.path());

    let mut startup = Box::pin(state.get_corsa_bridge());
    assert!(startup.as_mut().now_or_never().is_none());
    await_condition(|| root.path().join("backend.pids").exists());
    let pending = state
        .corsa_bridge
        .read()
        .clone()
        .expect("startup owner must be retained");
    assert!(
        !state.has_corsa_bridge(),
        "pending IPC is not an initialized bridge"
    );
    drop(startup);
    assert!(pending.is_draining());
    // This must resolve in the first poll without queueing another handshake
    // or creating a second backend while the first still owns uncancellable IPC.
    assert!(matches!(
        state.get_corsa_bridge().now_or_never(),
        Some(None)
    ));
    assert!(Arc::ptr_eq(
        &pending,
        state.corsa_bridge.read().as_ref().unwrap()
    ));
    assert_eq!(
        std::fs::read_to_string(root.path().join("backend.pids"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    assert!(
        !state
            .corsa_init_failed
            .load(std::sync::atomic::Ordering::SeqCst)
    );

    cleanup.stop();
    await_condition(|| !pending.is_draining());
    // The deliberate backend failed on release. Retry after its drain is
    // allowed on the same worker, so cancellation never poisons the failure latch.
    let mut retry = Box::pin(state.get_corsa_bridge());
    assert!(retry.as_mut().now_or_never().is_none());
    await_condition(|| {
        std::fs::read_to_string(root.path().join("backend.pids"))
            .is_ok_and(|pids| pids.lines().count() == 2)
    });
    assert!(Arc::ptr_eq(
        &pending,
        state.corsa_bridge.read().as_ref().unwrap()
    ));
    assert!(
        !state
            .corsa_init_failed
            .load(std::sync::atomic::Ordering::SeqCst)
    );
    drop(retry);
    cleanup.stop();
    await_condition(|| !pending.is_draining());
}

#[test]
fn returned_startup_timeout_keeps_the_existing_failure_latch_and_fallback() {
    let (root, state) = held_backend(1_000);
    let _cleanup = BackendCleanup(root.path());
    assert!(futures::executor::block_on(state.get_corsa_bridge()).is_none());
    assert!(state.corsa_bridge.read().is_none());
    assert!(
        state
            .corsa_init_failed
            .load(std::sync::atomic::Ordering::SeqCst)
    );
    assert!(
        state
            .corsa_init_failure()
            .unwrap()
            .contains("spawn timed out after 1000ms")
    );
    assert!(matches!(
        state.get_corsa_bridge().now_or_never(),
        Some(None)
    ));
    assert_eq!(
        std::fs::read_to_string(root.path().join("backend.pids"))
            .unwrap()
            .lines()
            .count(),
        1
    );
}

#[test]
fn superseded_startup_failure_cannot_poison_the_replacement_workspace() {
    let (root, state) = held_backend(60_000);
    let cleanup = BackendCleanup(root.path());
    let mut startup = Box::pin(state.get_corsa_bridge());
    assert!(startup.as_mut().now_or_never().is_none());
    await_condition(|| root.path().join("backend.pids").exists());
    let replacement = tempfile::tempdir().unwrap();
    state.set_workspace_root(replacement.path().to_owned());
    cleanup.stop();
    assert!(futures::executor::block_on(startup).is_none());
    assert!(state.corsa_bridge.read().is_none());
    assert!(
        !state
            .corsa_init_failed
            .load(std::sync::atomic::Ordering::SeqCst)
    );
    assert!(state.corsa_init_failure().is_none());
    assert_eq!(
        state.get_workspace_root().as_deref(),
        Some(replacement.path())
    );
}
