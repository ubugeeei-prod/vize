use super::{Command, native, retain_packet};
use sha2::{Digest, Sha256};

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
    assert_complete_stdout(&stdout, &native);
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

fn assert_complete_stdout(stdout: &str, native: &std::path::Path) {
    // libtest owns only elapsed time. Validate that token, then compare every
    // other byte, including both whole native packets and the test inventory.
    let summary_prefix =
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in ";
    let duration = stdout
        .lines()
        .nth(6)
        .and_then(|line| line.strip_prefix(summary_prefix))
        .and_then(|line| line.strip_suffix('s'))
        .expect("complete one-test/four-filtered libtest summary");
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
        "\nrunning 1 test\n{broken}\n{repaired}\ntest actual_native_syntax_and_semantic_errors_survive_and_repair_clears_only_the_semantic_error ... ok\n\n{summary_prefix}{duration}s\n\n"
    );
    assert_eq!(stdout, expected, "complete native handoff stdout");
}
