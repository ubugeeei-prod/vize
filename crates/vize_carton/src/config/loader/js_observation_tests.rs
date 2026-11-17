//! The config child transport, and configs that leave work behind.

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc,
    time::{Duration, Instant},
};

use super::{
    js_observation::{EXIT_GRACE, ObservationFailure, evaluate},
    load_lsp_config_snapshot, load_lsp_config_snapshot_observed,
    try_load_project_config_with_source,
};

/// Fail instead of hanging the suite when a host waits on a lingering child.
fn within<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(work());
    });
    receiver
        .recv_timeout(Duration::from_secs(60))
        .expect("the config host kept waiting on its child")
}

fn node(script: &str) -> Command {
    let mut command = Command::new("node");
    command
        .arg("-e")
        .arg(script)
        .arg("--")
        .arg("config-fixture");
    command
}

fn is_running(pid: u32) -> bool {
    let probe = Command::new("node")
        .arg("-e")
        .arg(crate::cstr!(
            "try {{ process.kill({pid}, 0); process.exit(1); }} catch (error) {{ process.exit(error.code === 'ESRCH' ? 0 : 2); }}"
        ))
        .status()
        .unwrap();
    !probe.success()
}

fn failure(error: &std::io::Error) -> &ObservationFailure {
    error
        .get_ref()
        .unwrap()
        .downcast_ref::<ObservationFailure>()
        .unwrap()
}

#[test]
fn drains_stderr_while_waiting_for_the_phase_ack_and_preserves_large_final_payload() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("policy.mjs");
    let encoded = serde_json::to_string(&[&path]).unwrap();
    let mut command = node(&crate::cstr!(
        r#"
const frame = Buffer.from({encoded:?});
process.stderr.write('e'.repeat(100000));
process.stdout.write(`B${{frame.length}}\n`);
process.stdout.write(frame);
process.stdin.once('data', ack => {{
  if (ack.length !== 1 || ack[0] !== 6) process.exit(2);
  process.stdout.end('R\n' + 'x'.repeat(100000));
  process.stdin.destroy();
}});
"#
    ));
    let mut observed = Vec::new();
    let output = evaluate(
        &mut command,
        Some(&mut |paths| observed.extend_from_slice(paths)),
    )
    .unwrap()
    .into_output()
    .unwrap();
    assert!(output.status.success());
    assert_eq!(observed, vec![path]);
    assert_eq!(output.stdout, vec![b'x'; 100000]);
    assert_eq!(output.stderr, vec![b'e'; 100000]);
}

#[test]
fn rejects_truncated_or_oversized_phases_and_reaps_the_child() {
    for frame in [
        "B65537\n",
        "B-1\n",
        "Bwat\n",
        "B10\n[]",
        "B1",
        "",
        "B5\n[\".\"]",
    ] {
        let mut command = node(&crate::cstr!(
            "process.stderr.write(\"malformed-specificreason\\n\"); process.stdout.write({frame:?});"
        ));
        let mut observed: Vec<PathBuf> = Vec::new();
        let Err(error) = evaluate(
            &mut command,
            Some(&mut |paths| observed.extend_from_slice(paths)),
        ) else {
            panic!("accepted invalid config phase {frame:?}");
        };
        let failure = failure(&error);
        assert_eq!(
            failure.stderr.as_ref().unwrap().as_slice(),
            b"malformed-specificreason\n",
            "lost stderr for {frame:?}"
        );
        assert!(failure.status.is_ok());
        assert!(observed.is_empty());
    }
}

#[test]
fn exit_before_final_payload_retains_actual_status_and_stderr() {
    let mut command = node("console.error('specificreason 日本語'); process.exit(17);");
    let Err(error) = evaluate(&mut command, Some(&mut |_| panic!("no phase emitted"))) else {
        panic!("accepted a child without a result");
    };
    let failure = failure(&error);
    assert_eq!(failure.status.as_ref().unwrap().code(), Some(17));
    assert_eq!(
        failure.stderr.as_ref().unwrap().as_slice(),
        "specificreason 日本語\n".as_bytes()
    );
    assert_eq!(failure.cause.kind(), std::io::ErrorKind::UnexpectedEof);
    let status = if cfg!(windows) {
        "exit code: 17"
    } else {
        "exit status: 17"
    };
    assert_eq!(
        crate::cstr!("{error}"),
        crate::cstr!(
            "node failed to load config: specificreason 日本語 ({status}; observation transport: failed to fill whole buffer)"
        )
    );
}

#[test]
fn startup_exception_before_finally_retains_actual_diagnostic_and_status() {
    // Independent Node execution owns the complete default exception packet,
    // including runtime-specific internal stack frames. Keep the actual throw
    // and argv identical; this oracle never calls the observation transport.
    let mut direct = node("throw new Error('startup-specificreason');");
    direct
        .arg("--observe-config-inputs")
        .stdin(std::process::Stdio::piped());
    let baseline = direct.output().unwrap();
    assert_eq!(baseline.status.code(), Some(1));
    assert_eq!(baseline.stdout.as_slice(), b"");
    let stderr = std::str::from_utf8(&baseline.stderr).unwrap();
    // Node emits CRLF on Windows. Parse only this independent diagnostic
    // header as lines; the observation transport must still preserve every
    // byte of the original packet, as asserted below.
    let mut diagnostic_lines = stderr.lines();
    assert_eq!(diagnostic_lines.next(), Some("[eval]:1"));
    assert_eq!(
        diagnostic_lines.next(),
        Some("throw new Error('startup-specificreason');")
    );
    assert_eq!(diagnostic_lines.next(), Some("^"));
    assert_eq!(diagnostic_lines.next(), Some(""));
    assert_eq!(
        diagnostic_lines.next(),
        Some("Error: startup-specificreason")
    );

    let mut command = node("throw new Error('startup-specificreason');");
    let Err(error) = evaluate(&mut command, Some(&mut |_| panic!("no phase emitted"))) else {
        panic!("accepted a child without a result");
    };
    let failure = failure(&error);
    assert_eq!(command.get_program(), direct.get_program());
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        direct.get_args().collect::<Vec<_>>()
    );
    assert_eq!(command.get_current_dir(), direct.get_current_dir());
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        direct.get_envs().collect::<Vec<_>>()
    );
    assert_eq!(failure.status.as_ref().unwrap(), &baseline.status);
    assert_eq!(failure.stderr.as_ref().unwrap(), &baseline.stderr);
    assert_eq!(failure.cause.kind(), std::io::ErrorKind::UnexpectedEof);
    assert_eq!(
        crate::cstr!("{}", failure.cause),
        "failed to fill whole buffer"
    );
    assert_eq!(
        crate::cstr!("{error}"),
        crate::cstr!(
            "node failed to load config: {} ({}; observation transport: failed to fill whole buffer)",
            stderr
                .strip_suffix(if cfg!(windows) { "\r\n" } else { "\n" })
                .unwrap(),
            baseline.status
        )
    );
}

#[test]
fn callback_panic_kills_and_reaps_the_child() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("child.pid");
    let encoded = serde_json::to_string(&[&path]).unwrap();
    let mut command = node(&crate::cstr!(
        r#"
const fs = require('node:fs');
const paths = {encoded};
fs.writeFileSync(paths[0], String(process.pid));
process.stderr.write('e'.repeat(100000));
const frame = Buffer.from(JSON.stringify(paths));
process.stdout.write(`B${{frame.length}}\n`);
process.stdout.write(frame);
setInterval(() => {{}}, 1000);
"#
    ));
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = evaluate(
            &mut command,
            Some(&mut |paths| {
                assert_eq!(paths, std::slice::from_ref(&path));
                panic!("configuration observer panic");
            }),
        );
    }));
    assert!(panic.is_err());
    let pid = std::fs::read_to_string(path)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    assert!(
        !is_running(pid),
        "configuration child {pid} survived or was not reaped"
    );
}

#[test]
fn a_child_that_keeps_running_after_its_result_line_is_stopped_after_the_grace() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("child.pid");
    let mut command = node(&crate::cstr!(
        r#"
require('node:fs').writeFileSync({path:?}, String(process.pid));
process.stdout.write('{{"value":1}}\n');
setInterval(() => {{}}, 1000);
"#,
        path = path
    ));
    let started = Instant::now();
    let child = within(move || evaluate(&mut command, None).unwrap());
    let elapsed = started.elapsed();
    assert!(child.stopped_after_result);
    assert!(!child.status.success());
    assert_eq!(child.stdout, br#"{"value":1}"#);
    assert!(elapsed >= EXIT_GRACE, "{elapsed:?}");
    let pid = std::fs::read_to_string(path)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    assert!(!is_running(pid), "configuration child {pid} survived");
}

#[test]
fn a_grandchild_holding_the_pipes_never_blocks_the_result() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("grandchild.pid");
    let mut command = node(&crate::cstr!(
        r#"
const {{ spawn }} = require('node:child_process');
const grandchild = spawn(process.execPath, ['-e', 'setInterval(() => {{}}, 1000)'], {{ stdio: 'inherit' }});
require('node:fs').writeFileSync({path:?}, String(grandchild.pid));
process.stderr.write('child-specificreason\n');
process.stdout.write('{{"value":2}}\n', () => process.exit(3));
"#,
        path = path
    ));
    let started = Instant::now();
    let output = within(move || evaluate(&mut command, None).unwrap().into_output().unwrap());
    let grandchild = std::fs::read_to_string(path)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let _ = Command::new("node")
        .arg("-e")
        .arg(crate::cstr!("process.kill({grandchild})"))
        .status();
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(output.stdout, br#"{"value":2}"#);
    assert_eq!(output.stderr, b"child-specificreason\n");
    // Stdout was never read to EOF; stderr waited at most the grace.
    assert!(
        started.elapsed() < EXIT_GRACE * 4,
        "{:?}",
        started.elapsed()
    );
}

/// A config left running, which records whether it exited by itself.
fn lingering_config(root: &Path, leftover: &str) -> (PathBuf, PathBuf) {
    let marker = root.join("exited");
    let path = root.join("vite.config.mjs");
    std::fs::write(
        &path,
        crate::cstr!(
            r#"import {{ writeFileSync }} from "node:fs";
process.on("exit", () => writeFileSync({marker:?}, "exited"));
{leftover}
export default {{ vue: {{ formatter: {{ printWidth: 40 + 2 }} }} }};
"#,
            marker = marker
        ),
    )
    .unwrap();
    (path, marker)
}

const LEFTOVERS: [&str; 2] = [
    "setInterval(() => {}, 1000);",
    r#"import { createServer } from "node:net";
const server = createServer().listen(0);
// Never settles: nothing closes the server.
const closed = new Promise((resolve) => server.on("close", resolve));
void closed;"#,
];

#[test]
fn cli_hosts_load_a_config_that_leaves_a_timer_or_server_running() {
    for leftover in LEFTOVERS {
        let root = tempfile::tempdir().unwrap();
        let (path, marker) = lingering_config(root.path(), leftover);
        let loaded = within(move || try_load_project_config_with_source(Some(&path)));
        let (config, _) = loaded.unwrap().document.into_config_and_features();
        assert_eq!(config.formatter.print_width, 42, "{leftover}");
        // It exited on its own once the result was out, not after the grace.
        assert_eq!(std::fs::read_to_string(marker).unwrap(), "exited");
    }
}

#[test]
fn editor_hosts_load_a_config_that_leaves_a_timer_or_server_running() {
    for leftover in LEFTOVERS {
        for observed in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let (path, marker) = lingering_config(root.path(), leftover);
            let snapshot = within(move || {
                if observed {
                    load_lsp_config_snapshot_observed(Some(&path), &mut |_| {})
                } else {
                    load_lsp_config_snapshot(Some(&path))
                }
            });
            assert!(snapshot.valid, "{leftover}: {:?}", snapshot.error);
            assert_eq!(snapshot.config.formatter.print_width, 42);
            assert_eq!(std::fs::read_to_string(marker).unwrap(), "exited");
        }
    }
}

/// The child's exit status behind a failed evaluation.
fn exit_code(error: &(dyn std::error::Error + 'static)) -> Option<i32> {
    let error = error.downcast_ref::<std::io::Error>()?.get_ref()?;
    match error.downcast_ref::<ObservationFailure>() {
        Some(failure) => failure.status.as_ref().ok()?.code(),
        None => error
            .downcast_ref::<super::failure_identity::NodeEvaluationFailure>()?
            .status()
            .code(),
    }
}

#[test]
fn a_config_that_never_settles_fails_on_every_host_instead_of_hanging() {
    for observed in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("vite.config.mjs");
        std::fs::write(&path, "export default new Promise(() => {});\n").unwrap();
        let code = within(move || {
            let mut observe = |_: &[PathBuf]| {};
            let mut before_read: super::BeforeConfigRead<'_> =
                observed.then_some(&mut observe as &mut dyn FnMut(&[PathBuf]));
            super::js::parse_js_config_observed(
                &path,
                &mut Vec::new(),
                &mut false,
                &mut before_read,
                None,
                false,
            )
            .map(|_| ())
            .map_err(|error| exit_code(error.as_ref()))
        });
        // Node's own exit for a top-level await that can never settle.
        assert_eq!(code, Err(Some(13)), "observed={observed}");
    }
}

#[test]
fn plugin_inspection_finishes_while_the_config_leaves_a_timer_running() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("vite.config.mjs");
    std::fs::write(
        &path,
        "setInterval(() => {}, 1000);\nexport default { vue: {} };\n",
    )
    .unwrap();
    let inspected = within(move || super::inspect_config_plugins(Some(&path)));
    assert!(inspected.is_ok(), "{inspected:?}");
}
