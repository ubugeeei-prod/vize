//! Passive artifact controls; these are not Corsa or diagnostic correctness proof.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "test-only JSON and filesystem controls use standard buffers"
)]

use std::{fs, path::PathBuf, process::Command};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[path = "support/lsp_process/passive.rs"]
#[allow(
    dead_code,
    reason = "the original LSP harness uses this shared helper's constructor and error reporter"
)]
mod passive;
use passive::Passive;

#[path = "support/lsp_process/process_state.rs"]
#[allow(dead_code, reason = "the LSP helper owns platform dispatch")]
mod process_state;

fn expected(terminal: Value) -> Value {
    json!({
        "schema":"vize-lsp-passive-terminal-evidence-v1",
        "process":{
            "pid":17,"declaredExecutable":"NONEXISTENT-PASSIVE-CONTROL",
            "arguments":["lsp"],"cwd":null,
            "sourceSha":std::env::var("SOURCE_SHA").ok(),
            "harnessPackageVersion":env!("CARGO_PKG_VERSION"),
            "identityScope":"original spawned LSP PID and declared command; no executable-byte or backend-PID join",
            "extraIdentityProbe":false
        },
        "observationTiming":"persisted after original teardown and reader joins",
        "terminalEvidence":terminal
    })
}

fn capture(root: PathBuf) -> Passive {
    let mut command = Command::new("NONEXISTENT-PASSIVE-CONTROL");
    command.arg("lsp");
    Passive::with_root(root, 17, &command, true)
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn passive_identity_has_no_probe_and_retains_the_entire_owned_packet() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("capture");
    let capture = capture(root.clone());
    assert_eq!(
        std::fs::read_dir(case.path()).unwrap().count(),
        0,
        "construction must not touch the filesystem"
    );
    let terminal = json!({
        "protocol":[
            {"direction":"request","message":{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}},
            {"direction":"response","message":{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///App.vue","diagnostics":[]}}}
        ],
        "stderrBytes":[112,104,97,115,101,10],
        "stderrUtf8":"phase\n",
        "exitCode":null,"exitSuccess":false,
        "stdoutReaderJoined":true,"stderrReaderJoined":true
    });
    let expected_bytes = serde_json::to_vec_pretty(&expected(terminal.clone())).unwrap();
    capture.persist(terminal).unwrap();
    assert_eq!(
        std::fs::read(root.join("evidence.json")).unwrap(),
        expected_bytes
    );
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest,
        json!({
            "complete":true,"originalBytes":expected_bytes.len(),"persistedBytes":expected_bytes.len(),
            "limitBytes":4*1024*1024,"originalSha256":digest(&expected_bytes),"persistedSha256":digest(&expected_bytes),
            "representation":"decoded JSON messages and already-owned stderr bytes; not raw wire frames",
            "overflowMeaning":"incomplete observation, never an accepted whole packet"
        })
    );
}

#[test]
fn overflow_is_explicit_and_cannot_be_a_whole_packet() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("capture");
    let terminal = json!({"stderrUtf8":"x".repeat(5*1024*1024)});
    let entire = serde_json::to_vec_pretty(&expected(terminal.clone())).unwrap();
    capture(root.clone()).persist(terminal).unwrap();
    let prefix = std::fs::read(root.join("evidence.json")).unwrap();
    assert_eq!(prefix, entire[..4 * 1024 * 1024]);
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        manifest,
        json!({
            "complete":false,"originalBytes":entire.len(),"persistedBytes":4*1024*1024,
            "limitBytes":4*1024*1024,"originalSha256":digest(&entire),"persistedSha256":digest(&prefix),
            "representation":"decoded JSON messages and already-owned stderr bytes; not raw wire frames",
            "overflowMeaning":"incomplete observation, never an accepted whole packet"
        })
    );
}

#[test]
fn artifact_write_failure_is_reported_without_panicking_or_overwriting() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("existing-file");
    std::fs::write(&root, b"original unrelated bytes").unwrap();
    let result = capture(root.clone()).persist(json!({"exitSuccess":false}));
    assert_eq!(
        result.as_ref().err().map(std::io::Error::kind),
        Some(std::io::ErrorKind::AlreadyExists)
    );
    assert_eq!(std::fs::read(root).unwrap(), b"original unrelated bytes");
}

#[test]
fn deadline_write_failure_returns_to_the_owner_and_preserves_unrelated_bytes() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("existing-file");
    fs::write(&root, b"original unrelated bytes").unwrap();
    let mut capture = Passive::with_root(
        root.clone(),
        17,
        &Command::new("NONEXISTENT-PASSIVE-CONTROL"),
        false,
    );
    capture.record_exit_deadline(
        17,
        std::time::Duration::from_secs(5),
        true,
        json!({"originalFailure":"original deadline expired"}),
    );
    assert!(capture.retains_terminal());
    assert_eq!(fs::read(root).unwrap(), b"original unrelated bytes");
}

#[test]
fn failure_only_capture_writes_nothing_on_success() {
    let case = tempfile::tempdir().unwrap();
    let capture = Passive::with_root(
        case.path().join("capture"),
        17,
        &Command::new("NONEXISTENT-PASSIVE-CONTROL"),
        false,
    );
    assert!(!capture.retains_terminal());
    capture
        .persist(json!({"exitSuccess":true,"protocol":[]}))
        .unwrap();
    assert_eq!(fs::read_dir(case.path()).unwrap().count(), 0);
}

#[test]
#[cfg(unix)]
fn deadline_capture_precedes_owner_kill_and_preserves_both_owned_packets() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("capture");
    let mut command = Command::new("sh");
    command
        .args(["-c", "read line"])
        .stdin(std::process::Stdio::piped());
    let mut child = command.spawn().unwrap();
    let mut capture = Passive::with_root(root.clone(), child.id(), &command, false);
    let owned = json!({
        "protocol":[{"direction":"request","message":{"jsonrpc":"2.0","method":"exit"}}],
        "stderrBytes":[],"stdoutReaderJoined":null,"stderrReaderJoined":null
    });
    capture.record_exit_deadline(
        child.id(),
        std::time::Duration::from_secs(5),
        true,
        owned.clone(),
    );
    let alive = child.try_wait().unwrap();
    // Only the existing owner kills/reaps; passive observation cannot do so.
    child.kill().unwrap();
    let status = child.wait().unwrap();
    assert_eq!(alive, None);
    assert!(capture.retains_terminal());
    let raw = fs::read(root.join("exit-deadline.json")).unwrap();
    let deadline: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(deadline["process"]["pid"], child.id());
    assert_eq!(deadline["originalTimeoutMillis"], 5000);
    assert_eq!(deadline["stdinOpen"], true);
    assert_eq!(deadline["ownedEvidenceAtDeadline"], owned);
    assert_eq!(
        deadline["stderrCompleteness"],
        "reader owns read_to_end bytes until EOF; final stderr follows original reader joins in evidence.json"
    );
    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join("exit-deadline-manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["complete"], true);
    assert_eq!(manifest["originalSha256"], digest(&raw));
    assert_eq!(manifest["persistedSha256"], digest(&raw));
    let terminal = json!({"originalFailure":"original deadline expired","exitSuccess":status.success(),"stderrBytes":[0,255,10]});
    capture.persist(terminal.clone()).unwrap();
    let packet: Value =
        serde_json::from_slice(&fs::read(root.join("evidence.json")).unwrap()).unwrap();
    assert_eq!(packet["terminalEvidence"], terminal);
}

#[test]
#[cfg(unix)]
fn complete_thread_and_fd_observations_retain_original_bytes() {
    use std::os::unix::ffi::OsStringExt;

    let case = tempfile::tempdir().unwrap();
    let process = case.path().join("17");
    let thread = process.join("task/19");
    fs::create_dir_all(&thread).unwrap();
    fs::create_dir(process.join("fd")).unwrap();
    std::os::unix::fs::symlink("pipe:[12345]", process.join("fd/0")).unwrap();
    let raw_target = b"/tmp/non-utf8-\xff";
    std::os::unix::fs::symlink(
        std::ffi::OsString::from_vec(raw_target.to_vec()),
        process.join("fd/1"),
    )
    .unwrap();
    for file in ["stat", "status", "wchan"] {
        fs::write(process.join(file), format!("process {file}\n")).unwrap();
    }
    let mut expected = serde_json::Map::new();
    for file in [
        "comm", "stat", "status", "wchan", "syscall", "stack", "children",
    ] {
        let bytes = format!("thread {file}\n").into_bytes();
        fs::write(thread.join(file), &bytes).unwrap();
        expected.insert(file.into(), json!({"complete":true,"bytes":bytes}));
    }
    let actual = process_state::from_proc(case.path(), 17);
    assert_eq!(
        actual["tasks"],
        json!({"complete":true,"entries":[{"name":"19","observation":expected}]})
    );
    assert_eq!(
        actual["fds"],
        json!({"complete":true,"entries":[
            {"name":"0","observation":{"target":"pipe:[12345]","targetBytes":b"pipe:[12345]"}},
            {"name":"1","observation":{"target":"/tmp/non-utf8-\u{fffd}","targetBytes":raw_target}}
        ]})
    );
    for file in ["stat", "status", "wchan"] {
        assert_eq!(
            actual[file],
            json!({"complete":true,"bytes":format!("process {file}\n").as_bytes()})
        );
    }
}

#[test]
fn missing_and_incomplete_observations_are_never_empty_successes() {
    let case = tempfile::tempdir().unwrap();
    let missing = process_state::from_proc(case.path(), 17);
    assert_eq!(missing["tasks"]["errorKind"], "NotFound");
    assert_eq!(missing["fds"]["errorKind"], "NotFound");
    let path = case.path().join("long");
    fs::write(&path, vec![b'x'; (64 * 1024) + 1]).unwrap();
    assert_eq!(
        process_state::read(&path),
        json!({"complete":false,"bytes":vec![b'x';64 * 1024]})
    );
    fs::create_dir(case.path().join("many")).unwrap();
    for item in ["1", "2"] {
        fs::write(case.path().join("many").join(item), b"").unwrap();
    }
    assert_eq!(
        process_state::entries(&case.path().join("many"), 1, |_| json!({"observed":true})),
        json!({"complete":false,"entries":[{"name":"1","observation":{"observed":true}}]})
    );
}

#[test]
fn aggregate_process_observation_budget_bounds_raw_bytes_and_stops_further_reads() {
    const LIMIT: usize = 4 * 1024 * 1024;
    const SCOPE: &str = "retained file/link/name bytes and file probes; excludes transient read_link allocation and JSON expansion";
    fn compact(value: &Value) -> Value {
        match value {
            Value::Object(fields) => Value::Object(
                fields
                    .iter()
                    .map(|(name, value)| {
                        let observed = if name == "bytes" || name == "targetBytes" {
                            let bytes = value
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|byte| u8::try_from(byte.as_u64().unwrap()).unwrap())
                                .collect::<Vec<_>>();
                            json!({"length":bytes.len(),"sha256":digest(&bytes)})
                        } else {
                            compact(value)
                        };
                        (name.clone(), observed)
                    })
                    .collect(),
            ),
            Value::Array(values) => Value::Array(values.iter().map(compact).collect()),
            _ => value.clone(),
        }
    }
    fn retained_bytes(value: &Value) -> usize {
        match value {
            Value::Object(fields) => fields
                .iter()
                .map(|(name, value)| {
                    if name == "bytes" || name == "targetBytes" {
                        value.as_array().unwrap().len()
                    } else {
                        retained_bytes(value)
                    }
                })
                .sum(),
            Value::Array(values) => values.iter().map(retained_bytes).sum(),
            _ => 0,
        }
    }

    let case = tempfile::tempdir().unwrap();
    let process = case.path().join("17");
    fs::create_dir_all(process.join("task")).unwrap();
    let payload = vec![b'x'; (64 * 1024) + 1];
    fs::write(process.join("stat"), &payload).unwrap();
    fs::write(process.join("wchan"), b"wait").unwrap();
    for task in 0..10 {
        let thread = process.join("task").join(format!("{task:02}"));
        fs::create_dir(&thread).unwrap();
        for file in [
            "comm", "stat", "status", "wchan", "syscall", "stack", "children",
        ] {
            fs::write(thread.join(file), &payload).unwrap();
        }
    }
    let missing = fs::File::open(process.join("status")).unwrap_err();
    let missing = json!({"errorKind":"NotFound","error":format!("{missing}")});
    let exhausted = json!({"complete":false,"reason":"observation byte budget exhausted"});
    let full = json!({"complete":false,"bytes":{
        "length":64*1024,"sha256":digest(&payload[..64*1024])
    }});
    // 63 full file probes, four wchan bytes and twenty directory-name bytes
    // leave 65,449 bytes for the last observed file. Seven files stay unread.
    let partial = json!({"complete":false,"reason":"observation byte budget exhausted",
        "bytes":{"length":65449,"sha256":digest(&payload[..65449])}
    });
    let expected_tasks = (0..10)
        .map(|task| {
            let mut fields = json!({
                "comm":full,"stat":full,"status":full,"wchan":full,
                "syscall":full,"stack":full,"children":full
            });
            if task == 8 {
                fields["children"] = partial.clone();
            } else if task == 9 {
                for file in [
                    "comm", "stat", "status", "wchan", "syscall", "stack", "children",
                ] {
                    fields[file] = exhausted.clone();
                }
            }
            json!({"name":format!("{task:02}"),"observation":fields})
        })
        .collect::<Vec<_>>();
    let actual = process_state::from_proc(case.path(), 17);
    assert!(retained_bytes(&actual) <= LIMIT);
    assert_eq!(
        compact(&actual),
        json!({
            "available":true,"pid":17,"stat":full,"status":missing,
            "wchan":{"complete":true,"bytes":{"length":4,"sha256":digest(b"wait")}},
            "tasks":{"complete":false,"reason":"observation byte budget exhausted","entries":expected_tasks},
            // The absent directory is skipped, rather than observed as NotFound.
            "fds":exhausted,
            "rawByteBudget":{"limitBytes":LIMIT,"chargedBytes":LIMIT,"exhausted":true,"scope":SCOPE}
        })
    );

    #[cfg(unix)]
    {
        let links = tempfile::tempdir().unwrap();
        let process = links.path().join("17");
        fs::create_dir_all(process.join("fd")).unwrap();
        std::os::unix::fs::symlink("whole-target", process.join("fd/0")).unwrap();
        let small = process_state::from_proc_with_limit(links.path(), 17, 4);
        // One byte names the FD; only the remaining three target bytes persist.
        assert_eq!(
            compact(&small),
            json!({
                "available":true,"pid":17,"stat":missing,"status":missing,"wchan":missing,
                "tasks":missing,
                "fds":{"complete":false,"reason":"observation byte budget exhausted","entries":[{
                    "name":"0","observation":{
                        "complete":false,"reason":"observation byte budget exhausted",
                        "originalBytes":12,"targetBytes":{"length":3,"sha256":digest(b"who")}
                    }
                }]},
                "rawByteBudget":{"limitBytes":4,"chargedBytes":4,"exhausted":true,"scope":SCOPE}
            })
        );
        assert_eq!(retained_bytes(&small), 3);
    }
    let empty = process_state::from_proc_with_limit(case.path(), 999, 0);
    assert_eq!(
        compact(&empty),
        json!({
            "available":true,"pid":999,"stat":exhausted,"status":exhausted,"wchan":exhausted,
            "tasks":exhausted,"fds":exhausted,
            "rawByteBudget":{"limitBytes":0,"chargedBytes":0,"exhausted":true,"scope":SCOPE}
        })
    );
}
