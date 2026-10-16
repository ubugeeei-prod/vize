//! Actual post-spawn RAII cleanup must not hold async startup admission.
use super::*;
use futures::FutureExt;
use serde_json::{Value, json};
use std::{path::Path, sync::mpsc};

const PROCESS: &str = r#"#!/usr/bin/env python3
import os,pathlib,subprocess,sys,time
root=pathlib.Path(os.getcwd())
if '--hold-stderr' in sys.argv:
    os.write(2,b'descendant-held\n')
    (root/'descendant.pid').write_text(str(os.getpid()))
    while not (root/'release-stderr').exists(): time.sleep(.005)
    os.write(2,b'descendant-release\n')
else:
    os.write(2,b'owned-start\n')
    (root/'owner.pid').write_text(str(os.getpid()))
    subprocess.Popen([sys.executable,__file__,'--hold-stderr'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL)
    while True: time.sleep(.005)
"#;
const FAULT: &str = "injected transport setup failure after actual owned process birth";

struct ReleaseOnDrop(std::path::PathBuf);
impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::write(self.0.join("release-stderr"), b"release\n");
    }
}

fn identity(pid: u32) -> Value {
    let process = Path::new("/proc").join(cstr!("{pid}").as_str());
    let stat = std::fs::read_to_string(process.join("stat")).unwrap();
    let fields: Vec<_> = stat
        .rsplit_once(") ")
        .unwrap()
        .1
        .split_whitespace()
        .collect();
    json!({"pid":pid,"birthTicks":fields[19],"parent":fields[1],"image":std::fs::read_link(process.join("exe")).unwrap(),"wholeStat":stat})
}

fn absent_before_cleanup(before: &Value) -> Value {
    let process = Path::new("/proc").join(cstr!("{}", before["pid"].as_u64().unwrap()).as_str());
    match std::fs::read_to_string(process.join("stat")) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => json!({"absent":true}),
        Err(error) => json!({"absent":false,"error":cstr!("{error}")}),
        Ok(stat) => {
            let fields: Vec<_> = stat
                .rsplit_once(") ")
                .unwrap()
                .1
                .split_whitespace()
                .collect();
            json!({"absent":fields[19] != before["birthTicks"].as_str().unwrap(),"wholeStat":stat})
        }
    }
}

#[test]
fn post_spawn_setup_drop_keeps_finalizer_poll_nonblocking_and_preserves_full_custody() {
    let root = tempfile::tempdir().unwrap();
    let _release = ReleaseOnDrop(root.path().to_owned());
    let executable = root.path().join("owned-setup");
    std::fs::write(&executable, PROCESS).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    let observer = ProcessObserver::default();
    let mut config = LspSpawnConfig::new(&executable).with_cwd(root.path());
    config.command = config.command.with_process_observer(observer.clone());
    let control = StartupControl::default();
    let child_control = control.clone();
    let child_observer = observer.clone();
    let (entered, ready) = mpsc::channel();
    let fixture_root = root.path().to_owned();
    let constructor = std::thread::spawn(move || {
        child_control.spawn_editor_with(config, child_observer, |config| {
            // A real SDK OwnedProcess is acquired before this injected setup
            // failure. Dropping it executes exact kill+wait+stderr custody.
            let owned = config.command.spawn_guarded(["--lsp", "--stdio"])?;
            let deadline = Instant::now() + Duration::from_secs(5);
            while !fixture_root.join("descendant.pid").is_file() {
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(5));
            }
            let descendant = std::fs::read_to_string(fixture_root.join("descendant.pid")).unwrap().parse::<u32>().unwrap();
            let witness = json!({"owner":identity(owned.process.terminal_observer().snapshot().expected_pid),"descendant":identity(descendant)});
            owned.process.terminal_observer().record_error(FAULT);
            entered.send(witness).unwrap();
            drop(owned);
            Err(corsa::CorsaError::Protocol(FAULT.into()))
        }).err().expect("actual injected setup failure")
    });
    let witness = ready.recv_timeout(Duration::from_secs(5)).unwrap();
    let until = Instant::now() + Duration::from_secs(1);
    loop {
        let receipts = observer.receipts();
        if receipts
            .first()
            .is_some_and(|receipt| receipt.reaped && !receipt.completed && !receipt.stderr_complete)
        {
            break;
        }
        assert!(
            Instant::now() < until,
            "real SDK Drop did not enter stderr drain: {receipts:?}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    let capture =
        std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR").map(std::path::PathBuf::from);
    if let Some(capture) = &capture {
        std::fs::create_dir_all(capture).unwrap();
        std::fs::write(
            capture.join("setup-drop-entered.json"),
            serde_json::to_vec_pretty(&witness).unwrap(),
        )
        .unwrap();
    }
    let mut finalizer = Box::pin(control.abort_pending(Instant::now() + Duration::from_secs(5)));
    let polled_at = Instant::now();
    let premature = finalizer.as_mut().now_or_never();
    let poll_time = polled_at.elapsed();
    let held_snapshot = control.lifetime_snapshot();
    let held_constructor_count = control.lock().spawning;
    // This release closes the descendant's inherited stderr normally. No
    // fixture PID kill participates in a passing cleanup.
    std::fs::write(root.path().join("release-stderr"), b"release\n").unwrap();
    let raw_setup_error = constructor.join().unwrap();
    let premature_ack = premature.is_some();
    let result = match premature {
        Some(result) => result,
        None => block_on(finalizer),
    };
    let snapshot = control.lifetime_snapshot();
    let until = Instant::now() + Duration::from_secs(5);
    let (owner_after, descendant_after) = loop {
        let owner = absent_before_cleanup(&witness["owner"]);
        let descendant = absent_before_cleanup(&witness["descendant"]);
        if (owner["absent"] == true && descendant["absent"] == true) || Instant::now() >= until {
            break (owner, descendant);
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let proof = json!({"before":witness,"pollNanos":poll_time.as_nanos(),"prematureAcknowledgment":premature_ack,"heldConstructorCount":held_constructor_count,"heldChildren":held_snapshot.pending.iter().map(crate::lsp_client::retirement::observation_json).collect::<Vec<_>>(),"rawSetupFailure":raw_setup_error,"finalizer":cstr!("{result:?}"),"children":snapshot.pending.iter().map(crate::lsp_client::retirement::observation_json).collect::<Vec<_>>(),"ownerAfter":owner_after,"descendantAfter":descendant_after});
    if let Some(capture) = capture {
        std::fs::write(
            capture.join("setup-drop-retirement.json"),
            serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
    }
    eprintln!("actual post-spawn setup Drop custody: {proof}");
    let test_pid = cstr!("{}", std::process::id());
    let owned_pid = cstr!("{}", witness["owner"]["pid"].as_u64().unwrap());
    assert_eq!(
        witness["owner"]["parent"].as_str(),
        Some(test_pid.as_str()),
        "{proof}"
    );
    assert_eq!(
        witness["descendant"]["parent"].as_str(),
        Some(owned_pid.as_str()),
        "{proof}"
    );
    assert_eq!(
        witness["owner"]["image"], witness["descendant"]["image"],
        "{proof}"
    );
    for role in ["owner", "descendant"] {
        assert!(
            witness[role]["birthTicks"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                > 0,
            "{proof}"
        );
    }
    assert!(
        poll_time < Duration::from_millis(250),
        "finalizer poll blocked async foreground: {proof}"
    );
    assert!(
        !premature_ack && held_constructor_count == 1,
        "admitted ownership was not awaited: {proof}"
    );
    assert!(!held_snapshot.pending[0].custody_complete());
    assert_eq!(
        raw_setup_error,
        cstr!("Cannot spawn native editor LSP: protocol error: {FAULT}")
    );
    assert!(result.is_ok(), "{proof}");
    assert_eq!(snapshot.pending.len(), 1);
    let observed = &snapshot.pending[0];
    assert!(observed.custody_complete(), "{proof}");
    assert_eq!(
        observed.expected_pid as u64,
        witness["owner"]["pid"].as_u64().unwrap()
    );
    assert_eq!(observed.raw_errors, [FAULT]);
    assert_eq!(
        observed.stderr_receipts.as_ref().unwrap()[0].stderr,
        b"owned-start\ndescendant-held\ndescendant-release\n"
    );
    let terminal = observed.terminal.as_ref().unwrap();
    use std::os::unix::process::ExitStatusExt;
    assert_eq!(terminal.status.signal(), Some(9));
    assert!(terminal.kill_requested && terminal.reaped && !terminal.timed_out);
    assert_eq!(owner_after["absent"], true, "{proof}");
    assert_eq!(descendant_after["absent"], true, "{proof}");
    assert!(!control.startup_custody_pending());
}

#[test]
fn sealed_constructor_handoff_retains_exact_owner_and_refuses_handshake() {
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("owned-handoff");
    std::fs::write(&executable, "#!/bin/sh\nexec cat > /dev/null\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    let observer = ProcessObserver::default();
    let mut config = LspSpawnConfig::new(&executable).with_cwd(root.path());
    config.command = config.command.with_process_observer(observer.clone());
    let control = StartupControl::default();
    let child_control = control.clone();
    let (entered, ready) = mpsc::channel();
    let (release, held) = mpsc::channel();
    let constructor = std::thread::spawn(move || {
        child_control
            .spawn_editor_with(config, observer, |config| {
                let client = block_on(LspClient::spawn(config))?;
                let corsa::process::ProcessOwnership::Owned(owner) = client.process_ownership()
                else {
                    panic!("actual stdio owner missing");
                };
                entered
                    .send(identity(owner.snapshot().expected_pid))
                    .unwrap();
                held.recv_timeout(Duration::from_secs(5)).unwrap();
                Ok(client)
            })
            .err()
            .expect("sealed constructor cannot enter its handshake")
    });
    let before = ready.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut close = Box::pin(control.abort_pending(Instant::now() + Duration::from_secs(5)));
    assert!(close.as_mut().now_or_never().is_none());
    assert!(control.lock().sealed);
    assert_eq!(control.lock().spawning, 1);
    release.send(()).unwrap();
    let refusal = constructor.join().unwrap();
    assert_eq!(refusal, "Native startup refused after final server close");
    assert!(block_on(close).is_ok());
    let snapshot = control.lifetime_snapshot();
    assert_eq!(snapshot.pending.len(), 1);
    assert!(snapshot.pending[0].custody_complete());
    assert_eq!(
        snapshot.pending[0].expected_pid as u64,
        before["pid"].as_u64().unwrap()
    );
    assert_eq!(absent_before_cleanup(&before)["absent"], true);
    assert!(!control.startup_custody_pending());
    assert_eq!(
        control.check_admission().unwrap_err(),
        "Native startup refused after final server close"
    );
    let proof = json!({"before":before,"refusal":refusal,"children":snapshot.pending.iter().map(crate::lsp_client::retirement::observation_json).collect::<Vec<_>>()});
    if let Some(capture) =
        std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR").map(std::path::PathBuf::from)
    {
        std::fs::create_dir_all(&capture).unwrap();
        std::fs::write(
            capture.join("sealed-constructor-handoff.json"),
            serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
    }
    eprintln!("actual sealed constructor handoff: {proof}");
}
