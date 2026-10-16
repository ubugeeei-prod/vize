use super::{Command, native, retain_packet};

/// A parser abort is a mandatory semantic phase, so the public fallback notice
/// must remain available for an actual transport/runtime failure later on.
#[test]
fn required_semantic_completion_does_not_report_runtime_degradation() {
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("running 1 test"),
        "the native corpus filter must execute its test: {output:?}"
    );
    assert!(
        !stderr.contains("Corsa CLI fast path unavailable"),
        "required same-config completion was reported as degradation: {stderr}"
    );
}
