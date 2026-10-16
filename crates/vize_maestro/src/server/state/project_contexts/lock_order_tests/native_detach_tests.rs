//! Pause real removal paths before deferred retirement, with owned IPC entered.
use super::{Phase, fixture, pause_at};
use futures::FutureExt;
use serde_json::{Value, json};
use std::{
    os::unix::fs::PermissionsExt,
    path::Path,
    sync::mpsc,
    time::{Duration, Instant},
};
use tower_lsp::lsp_types::{FileChangeType, FileEvent, Url};
use vize_l0::cstr;

struct EmergencyCleanup<'a>(&'a Path);
impl Drop for EmergencyCleanup<'_> {
    fn drop(&mut self) {
        if std::thread::panicking()
            && let Ok(pids) = std::fs::read_to_string(self.0.join("backend.pids"))
        {
            let _ = std::process::Command::new("kill")
                .args(pids.lines())
                .status();
        }
    }
}

fn entered(root: &Path) -> Value {
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(pids) = std::fs::read_to_string(root.join("backend.pids")) {
            assert_eq!(pids.lines().count(), 1);
            let pid: u32 = pids.trim().parse().unwrap();
            let process = Path::new("/proc").join(pid.to_string());
            if let (Ok(stat), Ok(image)) = (
                std::fs::read_to_string(process.join("stat")),
                std::fs::read_link(process.join("exe")),
            ) {
                let fields: Vec<_> = stat
                    .rsplit_once(") ")
                    .unwrap()
                    .1
                    .split_whitespace()
                    .collect();
                if image.file_name().is_some_and(|name| name == "cat") {
                    assert_eq!(fields[1].parse::<u32>().unwrap(), std::process::id());
                    assert!(fields[19].parse::<u64>().is_ok());
                    assert_ne!(fields[0], "Z");
                    return json!({"pid":pid,"birthTicks":fields[19],"parent":std::process::id(),"image":image,"wholeStat":stat});
                }
            }
        }
        assert!(
            Instant::now() < until,
            "the owned API handshake never entered"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn run(mode: &str) {
    let (root, state, _uri, context) = fixture();
    let _cleanup = EmergencyCleanup(root.path());
    let backend = root.path().join("held-corsa");
    std::fs::write(
        &backend,
        "#!/bin/sh\necho $$ >> backend.pids\nexec cat 3>&1 > /dev/null\n",
    )
    .unwrap();
    std::fs::set_permissions(&backend, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(root.path().join("tsconfig.json"), "{}").unwrap();
    std::fs::write(root.path().join("vize.config.json"), json!({"lsp":{"typecheck":true,"lint":false},"typeChecker":{"corsaPath":backend,"lspRequestTimeoutMs":60000}}).to_string()).unwrap();
    let child = if mode == "primary-watcher" {
        state.load_lsp_config(root.path());
        state.clone()
    } else {
        context.initialize(&state).unwrap()
    };
    let mut startup = Box::pin(child.get_corsa_bridge());
    assert!(startup.as_mut().now_or_never().is_none());
    let before = entered(root.path());
    drop(startup);
    let replacement = tempfile::tempdir().unwrap();
    if mode == "refresh" {
        state.set_workspace_root(replacement.path().to_owned());
    }
    let marker = Url::from_file_path(root.path().join("vize.config.json")).unwrap();
    let (paused, resume, install) = pause_at(Phase::AfterContextRemoval);
    let worker_state = state.clone();
    let operation = mode.to_owned();
    let worker = std::thread::spawn(move || {
        install();
        match operation.as_str() {
            "refresh" => worker_state.refresh_project_routes(),
            "watcher" | "primary-watcher" => {
                worker_state.observe_project_config_events(&[FileEvent {
                    uri: marker,
                    typ: FileChangeType::CHANGED,
                }]);
            }
            "drain" => worker_state.retire_project_contexts(),
            _ => panic!("unsupported native detach mode: {operation}"),
        }
    });
    paused.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(context.initialized_state().is_none());
    assert!(state.project_contexts.contexts.try_lock().is_some());
    let detached_slot_empty = child.corsa_bridge.read().is_none();
    let (reply, response) = mpsc::channel();
    let final_state = state.clone();
    let finalizer = std::thread::spawn(move || {
        let outcome = futures::executor::block_on(final_state.shutdown_corsa_project_contexts());
        reply.send(outcome).unwrap();
    });
    let outcome = response.recv_timeout(Duration::from_secs(5));
    let pid = before["pid"].as_u64().unwrap();
    let after = match std::fs::read_to_string(Path::new("/proc").join(pid.to_string()).join("stat"))
    {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            json!({"reaped":true,"observation":"absent"})
        }
        Err(error) => json!({"reaped":false,"error":error.to_string()}),
        Ok(stat) => {
            let fields: Vec<_> = stat
                .rsplit_once(") ")
                .unwrap()
                .1
                .split_whitespace()
                .collect();
            let birth = fields.get(19).copied();
            json!({"reaped":birth.is_some_and(|birth| birth.parse::<u64>().is_ok() && Some(birth) != before["birthTicks"].as_str()),"wholeStat":stat})
        }
    };
    let pids_after = std::fs::read_to_string(root.path().join("backend.pids")).unwrap();
    let proof = json!({"mode":mode,"ownerKind":if mode == "primary-watcher" {"primary-root"} else {"project-context"},"cachedOwnerTransferredBeforeDeferredRetire":detached_slot_empty,"before":before,"afterFinalSnapshotBeforeDeferredRetire":after,"finalOutcome":cstr!("{outcome:?}"),"backendPidsAfter":pids_after});
    if let Some(capture) = std::env::var_os("VIZE_NATIVE_CONTEXT_TRANSFER_CAPTURE_DIR") {
        let capture = std::path::PathBuf::from(capture);
        std::fs::create_dir_all(&capture).unwrap();
        std::fs::write(
            capture.join(cstr!("{mode}.json").as_str()),
            serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
    }
    println!("removed context native custody: {proof}");
    // Resume deferred work only AFTER retaining the actual finalization witness.
    let _ = resume.send(());
    if outcome.is_err()
        && after["wholeStat"].as_str().is_some_and(|stat| {
            stat.rsplit_once(") ")
                .and_then(|(_, fields)| fields.split_whitespace().nth(19))
                == before["birthTicks"].as_str()
        })
    {
        // Only a recorded deadline failure permits emergency test cleanup.
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status();
    }
    worker.join().unwrap();
    finalizer.join().unwrap();
    assert!(outcome.unwrap().is_ok(), "{proof}");
    assert!(
        detached_slot_empty,
        "native custody must transfer before detaching: {proof}"
    );
    assert_eq!(
        after["reaped"], true,
        "an absent cache/ledger is not native reaping: {proof}"
    );
    assert_eq!(
        pids_after.lines().count(),
        1,
        "late callbacks cannot spawn: {proof}"
    );
}

#[test]
fn refresh_transfers_native_before_detach_and_deferred_retire() {
    run("refresh");
}

#[test]
fn watcher_transfers_native_before_detach_and_deferred_retire() {
    run("watcher");
}

#[test]
fn watcher_transfers_primary_native_before_detach_and_deferred_retire() {
    run("primary-watcher");
}

#[test]
fn drain_transfers_native_before_detach_and_deferred_retire() {
    run("drain");
}
