use super::{Command, check, fs, install, native, retain_packet, rows};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tracing::{Event, Metadata, Subscriber, field::Visit, span};

const CHILD: &str = "VIZE_TEST_HANDOFF_NOTICE_CHILD";
const TARGET: &str = "vize_canon::corsa::fallback";
const CLI_FAILURE: &str = "native CLI intentionally unavailable";

#[derive(Clone, Default)]
struct FallbackEvents(Arc<Mutex<Vec<BTreeMap<String, String>>>>);

struct Fields(BTreeMap<String, String>);

impl Visit for Fields {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.insert(field.name().to_owned(), value.to_owned());
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().to_owned(), format!("{value:?}"));
    }
}

impl Subscriber for FallbackEvents {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.target() == TARGET
    }

    fn new_span(&self, _: &span::Attributes<'_>) -> span::Id {
        span::Id::from_u64(1)
    }

    fn record(&self, _: &span::Id, _: &span::Record<'_>) {}
    fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}
    fn enter(&self, _: &span::Id) {}
    fn exit(&self, _: &span::Id) {}

    fn event(&self, event: &Event<'_>) {
        if !self.enabled(event.metadata()) {
            return;
        }
        let mut fields = Fields(BTreeMap::from([
            ("target".into(), event.metadata().target().into()),
            ("level".into(), event.metadata().level().to_string()),
        ]));
        event.record(&mut fields);
        self.0.lock().unwrap().push(fields.0);
    }
}

const OBSERVED_NATIVE_SHA256: &str =
    "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
const OBSERVED_EXIT_WARNINGS: &str =
    "error handling method 'exit': EOF\nerror handling method 'exit': EOF\n";

/// A parser abort is a mandatory semantic phase, so the public fallback notice
/// must remain available for an actual transport/runtime failure later on.
#[test]
fn required_semantic_completion_does_not_report_runtime_degradation() {
    // Negative controls must never expand the observed shutdown frontier.
    for (stderr, identity, expected) in [
        ("", "another-native-image", Ok(ShutdownObservation::Clean)),
        (
            OBSERVED_EXIT_WARNINGS,
            OBSERVED_NATIVE_SHA256,
            Ok(ShutdownObservation::KnownObservedExitWarnings),
        ),
        (
            OBSERVED_EXIT_WARNINGS,
            "another-native-image",
            Err("unrecognized native stderr or image"),
        ),
        (
            "error handling method 'exit': EOF\n",
            OBSERVED_NATIVE_SHA256,
            Err("unrecognized native stderr or image"),
        ),
        (
            "error handling method 'exit': EOF\nerror handling method 'exit': EOF\nerror handling method 'exit': EOF\n",
            OBSERVED_NATIVE_SHA256,
            Err("unrecognized native stderr or image"),
        ),
        (
            "error handling method 'exit': EOF\ncontext canceled\n",
            OBSERVED_NATIVE_SHA256,
            Err("unrecognized native stderr or image"),
        ),
        (
            "Corsa CLI fast path unavailable\n",
            OBSERVED_NATIVE_SHA256,
            Err("unrecognized native stderr or image"),
        ),
        (
            "error handling method 'exit': EOF\nerror handling method 'exit': EOF\nCorsa CLI fast path unavailable\n",
            OBSERVED_NATIVE_SHA256,
            Err("unrecognized native stderr or image"),
        ),
    ] {
        assert_eq!(observe_shutdown(stderr, identity), expected);
    }
    let Some(native) = native() else { return };
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "actual_native_syntax_and_semantic_errors_survive_and_repair_clears_only_the_semantic_error",
            "--nocapture",
        ])
        .env("CORSA_PATH", &native)
        .env("VIZE_TEST_REQUIRE_TSGO", "1")
        .env_remove("VIZE_SILENCE_CORSA_FALLBACK")
        .output()
        .unwrap();
    retain_packet(
        "semantic-handoff-child",
        &serde_json::json!({
            "native": native,
            "argv": ["--exact", "actual_native_syntax_and_semantic_errors_survive_and_repair_clears_only_the_semantic_error", "--nocapture"],
            "exitCode": output.status.code(),
            "wholeStdout": String::from_utf8(output.stdout.clone()).unwrap(),
            "wholeStderr": String::from_utf8(output.stderr.clone()).unwrap(),
        }),
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_complete_stdout(
        &stdout,
        &native,
        "actual_native_syntax_and_semantic_errors_survive_and_repair_clears_only_the_semantic_error",
    );
    let native_sha256 = Sha256::digest(std::fs::read(&native).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let shutdown_observation = observe_shutdown(&stderr, &native_sha256).unwrap_or_else(|reason| {
        panic!(
            "{reason}; complete child packet: {}",
            serde_json::json!({
                "native": native, "nativeSha256": native_sha256,
                "exitCode": output.status.code(),
                "wholeStdout": stdout, "wholeStderr": stderr,
            })
        )
    });
    retain_packet(
        "semantic-handoff-shutdown-observation",
        &serde_json::json!({
            "category": vize_l0::cstr!("{shutdown_observation:?}").as_str(),
            "wholeStderr": stderr,
            "nativeSha256": native_sha256,
            "followUp": "https://github.com/ubugeeei-prod/vize/pull/9239",
        }),
    );
    assert_eq!(
        shutdown_observation,
        ShutdownObservation::Clean,
        "authenticated native shutdown must now be clean; complete stdout={stdout:?}; stderr={stderr:?}"
    );
}

#[derive(Debug, PartialEq, Eq)]
enum ShutdownObservation {
    Clean,
    KnownObservedExitWarnings,
}

fn observe_shutdown(
    stderr: &str,
    native_sha256: &str,
) -> Result<ShutdownObservation, &'static str> {
    match (stderr, native_sha256) {
        ("", _) => Ok(ShutdownObservation::Clean),
        // Precisely the physical Linux x64 native 7.0.2 image observed in run
        // 38925869757. These two Batch exits are a known current limitation,
        // NOT desired clean shutdown. #9239 owns ACK -> EOF without filtering.
        (OBSERVED_EXIT_WARNINGS, OBSERVED_NATIVE_SHA256) => {
            Ok(ShutdownObservation::KnownObservedExitWarnings)
        }
        _ => Err("unrecognized native stderr or image"),
    }
}

fn assert_complete_stdout(stdout: &str, native: &Path, test_name: &str) {
    // libtest owns only elapsed time. Validate that token, then compare every
    // other byte, including both whole native packets and the test inventory.
    let summary_prefix =
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in ";
    let duration = stdout
        .lines()
        .nth(6)
        .and_then(|line| line.strip_prefix(summary_prefix))
        .and_then(|line| line.strip_suffix('s'))
        .expect("complete one-test/six-filtered libtest summary");
    let (seconds, hundredths) = duration.split_once('.').unwrap();
    assert_eq!(hundredths.len(), 2);
    assert!(seconds.bytes().all(|byte| byte.is_ascii_digit()));
    assert!(hundredths.bytes().all(|byte| byte.is_ascii_digit()));
    seconds.parse::<u64>().unwrap();
    let syntax = serde_json::json!([
        "post-request.d.ts",
        1293,
        0,
        1128,
        1,
        "Declaration or statement expected.",
    ]);
    let semantic = serde_json::json!([
        "Injected.vue",
        1,
        6,
        2322,
        1,
        "Type 'string' is not assignable to type 'number'.",
    ]);
    let packet = |broken, diagnostics| {
        serde_json::json!({
            "broken": broken, "diagnostics": diagnostics, "native": native,
            "originalCliStdout": "post-request.d.ts(1294,1): error TS1128: Declaration or statement expected.\n",
        })
    };
    let broken = packet(true, vec![semantic, syntax.clone()]);
    let repaired = packet(false, vec![syntax]);
    let expected = format!(
        "\nrunning 1 test\n{broken}\n{repaired}\ntest {test_name} ... ok\n\n{summary_prefix}{duration}s\n\n"
    );
    assert_eq!(stdout, expected, "complete native handoff stdout");
}

/// A parser abort is a mandatory semantic phase, so the public fallback notice
/// must remain available for an actual transport/runtime failure later on.
#[test]
fn structured_handoff_preserves_runtime_degradation_signal() {
    let Some(native) = native() else { return };
    if let Some(mode) = std::env::var_os(CHILD) {
        assert_eq!(mode, "1");
        let events = FallbackEvents::default();
        // A child-only global subscriber also observes worker threads, without
        // capturing unrelated tests running concurrently in the parent process.
        tracing::subscriber::set_global_default(events.clone()).unwrap();
        super::actual_native_syntax_and_semantic_errors_survive_and_repair_clears_only_the_semantic_error();
        assert_eq!(*events.0.lock().unwrap(), Vec::new());
        assert_real_runtime_degradation(&native, &events);
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "handoff_notice::structured_handoff_preserves_runtime_degradation_signal",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env("CORSA_PATH", &native)
        .env("VIZE_TEST_REQUIRE_TSGO", "1")
        .env_remove("VIZE_SILENCE_CORSA_FALLBACK")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    retain_packet(
        "handoff-notice-child",
        &serde_json::json!({"exitCode":output.status.code(),
            "wholeStdout":output.stdout,"wholeStderr":output.stderr}),
    );
    // Native stderr has a different producer from Vize's structured warning.
    // Keep every byte observable; this test does not certify native shutdown.
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_complete_stdout(
        &stdout,
        &native,
        "handoff_notice::structured_handoff_preserves_runtime_degradation_signal",
    );
    let notices = stderr
        .lines()
        .filter(|line| line.starts_with("\u{1b}[33mwarning:\u{1b}[0m corsa: "))
        .collect::<Vec<_>>();
    assert_eq!(
        notices,
        [format!(
            "\u{1b}[33mwarning:\u{1b}[0m corsa: Corsa CLI fast path unavailable; degraded to the slower project-session API (check failure: corsa error (exit code 42): {CLI_FAILURE}). Type checking continues on a slower path."
        )]
    );
}

fn wrapper(root: &Path, native: &Path, transport: Option<i32>) -> PathBuf {
    let bin = root.join(".bin");
    fs::create_dir_all(&bin).unwrap();
    let native = native.to_string_lossy().replace('\'', "'\"'\"'");
    let command = if let Some(code) = transport {
        let peer = bin.join("diagnostic-refusal.ts");
        fs::write(&peer, include_str!("diagnostic_refusal.ts")).unwrap();
        let peer = peer.to_string_lossy().replace('\'', "'\"'\"'");
        let capture = bin.join("refusal.json");
        let capture = capture.to_string_lossy().replace('\'', "'\"'\"'");
        format!(
            "case \"$1\" in --api) exec node '{peer}' '{capture}' '{code}' \"$@\";; esac\nexec '{native}' \"$@\""
        )
    } else {
        format!(
            "case \"$1\" in --api|--lsp) exec '{native}' \"$@\";; esac\nprintf '%s\\n' '{CLI_FAILURE}' >&2\nexit 42"
        )
    };
    let path = bin.join("native-handoff-control");
    fs::write(&path, format!("#!/bin/sh\n{command}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn assert_real_runtime_degradation(native: &Path, events: &FallbackEvents) {
    let project = tempfile::tempdir().unwrap();
    install(project.path(), true);
    let wrapper = wrapper(project.path(), native, None);
    let result = check(project.path(), &wrapper).expect("real native session must complete");
    assert!(!result.success);
    assert_eq!(
        rows(project.path(), &result),
        vec![
            (
                "Injected.vue".into(),
                1,
                6,
                Some(2322),
                1,
                "Type 'string' is not assignable to type 'number'.".into()
            ),
            (
                "post-request.d.ts".into(),
                1293,
                0,
                Some(1128),
                1,
                "Declaration or statement expected.".into()
            ),
        ]
    );
    let expected = BTreeMap::from([
        ("target".into(), TARGET.into()),
        ("level".into(), "WARN".into()),
        ("fallback".into(), "CliToSession".into()),
        ("cause".into(), "check".into()),
        (
            "error".into(),
            format!("corsa error (exit code 42): {CLI_FAILURE}"),
        ),
        (
            "message".into(),
            "Corsa CLI fast path unavailable; degraded to the slower project-session API".into(),
        ),
    ]);
    let actual = events.0.lock().unwrap().clone();
    retain_packet("handoff-fallback-events", &serde_json::json!(actual));
    assert_eq!(actual, [expected]);
}

#[test]
fn cancelled_and_failed_semantic_requests_cannot_report_completion() {
    let Some(native) = native() else { return };
    for (code, message) in [
        (-32800, "context canceled"),
        (-32603, "semantic request failed"),
    ] {
        let project = tempfile::tempdir().unwrap();
        install(project.path(), true);
        let wrapper = wrapper(project.path(), &native, Some(code));
        let error = check(project.path(), &wrapper)
            .expect_err("a failed semantic response must not complete the parser handoff");
        let vize_canon::CorsaError::CorsaExecution {
            exit_code,
            message: actual,
        } = error
        else {
            panic!("semantic refusal must be an execution failure: {error:#?}");
        };
        assert_eq!(exit_code, 1);
        assert_eq!(
            actual.as_str(),
            format!(
                "corsa error (exit code 1): Corsa CLI semantic diagnostics are incomplete after a syntax error:\npost-request.d.cts(1294,1): error TS1128: Declaration or statement expected.\nSame-config native diagnostic completion failed: corsa error (exit code -1): Failed to request Corsa project diagnostics: rpc error {code}: {message}"
            )
        );
        let packet: serde_json::Value =
            serde_json::from_slice(&fs::read(project.path().join(".bin/refusal.json")).unwrap())
                .unwrap();
        retain_packet(&format!("handoff-semantic-refusal-{code}"), &packet);
        assert_eq!(packet["request"]["method"], "getDiagnosticsForProject");
        assert_eq!(
            packet["request"]["params"],
            serde_json::json!({"snapshot":"denied-snapshot", "project":"denied-project"})
        );
        assert_eq!(
            packet["response"],
            serde_json::json!({"jsonrpc":"2.0", "id":packet["request"]["id"], "error":{"code":code,"message":message}})
        );
    }
}
