//! Ordinary recovery offers complete immutable rows before exact pruning.
use super::*;
use serde_json::{Value, json};
use tracing::{Event, Metadata, Subscriber, span};

#[derive(Clone)]
struct Sink {
    control: StartupControl,
    rows: Arc<Mutex<Vec<Value>>>,
}
struct Receipt(Option<Value>);
impl tracing::field::Visit for Receipt {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "receipt" {
            self.0 = Some(serde_json::from_str(cstr!("{value:?}").as_str()).unwrap());
        }
    }
}
impl Subscriber for Sink {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, _: &span::Attributes<'_>) -> span::Id {
        span::Id::from_u64(1)
    }
    fn record(&self, _: &span::Id, _: &span::Record<'_>) {}
    fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}
    fn event(&self, event: &Event<'_>) {
        let mut receipt = Receipt(None);
        event.record(&mut receipt);
        if let Some(mut receipt) = receipt.0 {
            let snapshot = self.control.lifetime_snapshot();
            receipt["beforePrunePublishedCount"] = json!(snapshot.published);
            receipt["beforePrunePendingPids"] = json!(
                snapshot
                    .pending
                    .iter()
                    .map(|owner| owner.expected_pid)
                    .collect::<Vec<_>>()
            );
            self.rows.lock().unwrap().push(receipt);
        }
    }
    fn enter(&self, _: &span::Id) {}
    fn exit(&self, _: &span::Id) {}
}

fn spawn(control: &StartupControl, executable: &std::path::Path) -> (LspClient, Registration) {
    let observer = ProcessObserver::default();
    let mut config = LspSpawnConfig::new(executable);
    config.command = config.command.with_process_observer(observer.clone());
    control.spawn_editor(config, observer).unwrap()
}

fn recover_twice(control: &StartupControl) -> (Vec<u32>, u32) {
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("recovery-owner");
    std::fs::write(&executable, "#!/bin/sh\nexec cat > /dev/null\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    let (live, live_registration) = spawn(control, &executable);
    let corsa::process::ProcessOwnership::Owned(live_observer) = live.process_ownership() else {
        panic!("owned live fixture");
    };
    let live_pid = live_observer.snapshot().expected_pid;
    let mut retired = Vec::new();
    for index in 0..2 {
        let (client, registration) = spawn(control, &executable);
        let owner = client.process_ownership();
        let corsa::process::ProcessOwnership::Owned(observer) = &owner else {
            panic!("owned recovered fixture");
        };
        let pid = observer.snapshot().expected_pid;
        block_on(client.close()).unwrap();
        let raw_failure =
            block_on(client.request_without_params::<lsp_types::request::Shutdown>()).unwrap_err();
        let raw_failure = cstr!("{raw_failure}");
        assert_eq!(raw_failure, "process is closed: jsonrpc connection");
        crate::lsp_client::retirement::ownership_error(&owner, raw_failure.as_str());
        control.publish_completed(Some(cstr!("recovery {index}").as_str()));
        retired.push(pid);
        let snapshot = control.lifetime_snapshot();
        assert_eq!(snapshot.observed, index + 2);
        assert_eq!(snapshot.published, index + 1);
        assert_eq!(snapshot.pending.len(), 1);
        assert_eq!(snapshot.pending[0].expected_pid, live_pid);
        assert!(!snapshot.pending[0].custody_complete());
        drop(registration);
    }
    // Seal suppresses ordinary publication, reserving this final live owner's
    // exact complete row for the final typed caller, independently of logging.
    assert!(block_on(control.abort_pending(Instant::now() + Duration::from_secs(5))).is_ok());
    let final_snapshot = control.lifetime_snapshot();
    assert_eq!(final_snapshot.pending.len(), 1);
    assert_eq!(final_snapshot.pending[0].expected_pid, live_pid);
    assert!(final_snapshot.pending[0].custody_complete());
    control.publish_completed(None);
    assert_eq!(control.lifetime_snapshot().pending.len(), 1);
    drop(live_registration);
    drop(live);
    (retired, live_pid)
}

#[test]
fn two_actual_recoveries_publish_whole_raw_custody_before_prune_with_live_owner_retained() {
    let control = StartupControl::default();
    let rows = Arc::new(Mutex::new(Vec::new()));
    let sink = Sink {
        control: control.clone(),
        rows: Arc::clone(&rows),
    };
    let (retired, live) = tracing::subscriber::with_default(sink, || recover_twice(&control));
    let rows = rows.lock().unwrap();
    let recovered: Vec<_> = rows
        .iter()
        .filter(|row| row.get("retirementError").is_some())
        .collect();
    assert_eq!(recovered.len(), 2);
    for (index, row) in recovered.iter().enumerate() {
        assert_eq!(row["beforePrunePublishedCount"], index);
        let children = row["children"].as_array().unwrap();
        assert_eq!(children.len(), 1);
        let child = &children[0];
        assert_eq!(child["expectedPid"], retired[index]);
        assert_eq!(child["terminal"]["pid"], retired[index]);
        assert_eq!(child["terminal"]["exitCode"], 0);
        assert_eq!(child["terminal"]["reaped"], true);
        assert_eq!(child["terminal"]["killRequested"], false);
        assert_eq!(
            child["raw_errors"],
            json!(["process is closed: jsonrpc connection"])
        );
        assert_eq!(child["stderrCustody"]["receipts"][0]["bytes"], json!([]));
        assert_eq!(
            child["stderrCustody"]["receipts"][0]["stderrComplete"],
            true
        );
        assert!(
            row["beforePrunePendingPids"]
                .as_array()
                .unwrap()
                .contains(&json!(retired[index]))
        );
        assert!(
            row["beforePrunePendingPids"]
                .as_array()
                .unwrap()
                .contains(&json!(live))
        );
    }
    let proof = json!({"retiredPids":retired,"retainedLivePid":live,"diagnosticHandoffs":*rows});
    if let Some(capture) =
        std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR").map(std::path::PathBuf::from)
    {
        std::fs::create_dir_all(&capture).unwrap();
        std::fs::write(
            capture.join("ordinary-recovery-publication.json"),
            serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
    }
    eprintln!("actual ordinary recovery custody publication: {proof}");
}

#[test]
fn disabled_diagnostic_sink_still_prunes_only_completed_actual_recovery_metadata() {
    let control = StartupControl::default();
    tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
        recover_twice(&control);
    });
    let snapshot = control.lifetime_snapshot();
    assert_eq!(snapshot.observed, 3);
    assert_eq!(snapshot.published, 2);
    assert_eq!(snapshot.pending.len(), 1);
    assert!(snapshot.pending[0].custody_complete());
}
