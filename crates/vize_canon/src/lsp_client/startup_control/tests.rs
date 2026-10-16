//! A normally closed typed client may still have its startup registration.
use super::*;
use std::{
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};

#[test]
fn completed_editor_close_before_registration_drop_is_not_a_false_reap_failure() {
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("owned-stdio");
    std::fs::write(&executable, "#!/bin/sh\nexec cat > /dev/null\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    let observer = ProcessObserver::default();
    let mut config = LspSpawnConfig::new(&executable).with_cwd(root.path());
    config.command = config.command.with_process_observer(observer.clone());
    let control = StartupControl::default();
    let (client, registration) = control.spawn_editor(config, observer.clone()).unwrap();
    let unpublished = control.lifetime_snapshot();
    assert_eq!(unpublished.pending.len(), 1);
    assert!(!unpublished.pending[0].custody_complete());
    assert!(unpublished.completed.is_empty());
    block_on(client.close()).unwrap();
    // Completing after an earlier incomplete snapshot cannot acknowledge a row
    // that snapshot never contained. The next publication retains its terminal.
    control.acknowledge_publication(&unpublished.completed);
    assert_eq!(control.lifetime_snapshot().published, 0);
    assert!(
        !unpublished.pending[0].custody_complete(),
        "the earlier terminal and stderr snapshot is immutable"
    );
    assert!(control.lifetime_snapshot().pending[0].custody_complete());
    assert!(block_on(control.abort_pending(Instant::now() + Duration::from_secs(1))).is_ok());
    let receipts = observer.receipts();
    assert_eq!(receipts.len(), 1);
    assert!(receipts[0].completed && receipts[0].reaped);
    assert!(receipts[0].exit_status.unwrap().success());
    assert!(!receipts[0].kill_requested && !receipts[0].timed_out);
    assert!(receipts[0].stderr.is_empty() && receipts[0].stderr_complete);
    assert!(control.lock().first_failure.is_none());
    assert!(control.lock().pending.is_empty());
    drop(registration);
    assert!(block_on(control.abort_pending(Instant::now() + Duration::from_secs(1))).is_ok());
}

#[test]
#[ignore = "requires the pinned native runtime; the native Canon Actions gate executes it"]
fn failed_native_project_open_drop_preserves_exact_nonowning_terminal_custody() {
    use sha2::{Digest, Sha256};
    use std::{path::Path, process::Command};
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let explicit = std::env::var_os("CORSA_PATH");
    let executable = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: explicit.as_deref().map(Path::new),
            project_root: Some(repo),
        },
    )
    .unwrap()
    .canonicalize()
    .unwrap();
    let version = Command::new(&executable).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert_eq!(version.stdout, b"Version 7.0.2\n");
    assert!(version.stderr.is_empty());
    let root = tempfile::tempdir().unwrap();
    let absent = root.path().join("absent-project.json");
    let control = StartupControl::default();
    // This invokes the real bootstrap path. Native updateSnapshot returns no
    // projects for this absent config, so ProjectSession::open fails before a
    // CorsaProjectClient or retained project session can own the child.
    let failure = crate::lsp_client::session::spawn_project_session_with_control(
        executable.to_str().unwrap(),
        root.path(),
        &absent,
        Some(&control),
    )
    .err()
    .expect("actual project open must fail");
    let failure = match failure {
        crate::lsp_client::session::ProjectSessionSpawnError::Unavailable(message) => message,
        other => panic!("wrong actual native bootstrap failure variant: {other:?}"),
    };
    assert_eq!(
        failure,
        cstr!(
            "Failed to start Corsa API session: protocol error: project session did not resolve a project (project {})",
            crate::file_uri::path_to_typescript_name(&absent)
        )
    );
    assert!(
        control.lock().pending.is_empty(),
        "startup Registration dropped"
    );
    let observers = control.unpublished_observers();
    assert_eq!(observers.len(), 1);
    // These observers hold no SDK client, Child or transport. Keeping this Arc
    // alive must not prevent the failed SDK constructor's Drop from reaping.
    let observed = observers[0].wait_for_terminal(Instant::now() + Duration::from_secs(5));
    assert!(observed.custody_complete(), "{observed:?}");
    let terminal = observed.terminal.as_ref().unwrap();
    assert_eq!(terminal.pid, observed.expected_pid);
    assert!(terminal.reaped && !terminal.timed_out);
    assert_eq!(observed.context, "api");
    assert_eq!(
        observed.raw_errors,
        ["native project open failed: protocol error: project session did not resolve a project"]
    );
    #[cfg(target_os = "linux")]
    assert!(
        !Path::new(&cstr!("/proc/{}", observed.expected_pid)).exists(),
        "actual child must already be absent before finalization"
    );
    assert!(block_on(control.abort_pending(Instant::now() + Duration::from_secs(1))).is_ok());
    let snapshot = control.lifetime_snapshot();
    assert_eq!(snapshot.observed, 1);
    assert_eq!(snapshot.published, 0);
    assert_eq!(snapshot.pending.len(), 1);
    assert_eq!(snapshot.completed.len(), 1);
    assert!(snapshot.pending[0].custody_complete());
    let binary_sha: String = Sha256::digest(std::fs::read(&executable).unwrap())
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect();
    let receipt = json!({"executable":executable,"binarySha256":binary_sha,"versionStdout":version.stdout,"versionStderr":version.stderr,"openProject":absent,"rawBootstrapFailure":failure,"child":crate::lsp_client::retirement::observation_json(&snapshot.pending[0])});
    let capture = std::path::PathBuf::from(
        std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR")
            .expect("native gate capture directory"),
    );
    std::fs::create_dir_all(&capture).unwrap();
    std::fs::write(
        capture.join("failed-project-open-drop.json"),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
    eprintln!("actual native failed-open Drop custody: {receipt}");
    // Explicit typed publication precedes pruning, even without a subscriber.
    control.acknowledge_publication(&snapshot.completed);
    let published = control.lifetime_snapshot();
    assert_eq!(published.observed, 1);
    assert_eq!(published.published, 1);
    assert!(published.pending.is_empty());
}

#[cfg(target_os = "linux")]
mod setup_drop;

mod recovery_publication;
