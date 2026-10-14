use crate::validate_runtime_output;
use serde_json::json;
use std::process::Command;

#[test]
fn runtime_failure_preserves_non_utf8_child_stream_bytes() {
    for byte in [255, 254] {
        let output = Command::new("node")
            .args([
                "-e",
                &format!(
                    "require('node:fs').writeSync(1, Buffer.from([{byte},0,10,97])); require('node:fs').writeSync(2, Buffer.from([{byte},13,27])); process.exit(13)"
                ),
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(13));
        assert_eq!(output.stdout, [byte, 0, 10, 97]);
        assert_eq!(output.stderr, [byte, 13, 27]);
        let report =
            validate_runtime_output("vdom", "source", &json!([{}]), "code", &output).unwrap_err();
        assert_eq!(
            report,
            format!(
                "vdom: runtime child failed\nstatus: {}\nexit_code: Some(13)\nsignal: None\nstdout:\n\u{fffd}\0\na\nstdout_bytes: [{byte}, 0, 10, 97]\nstderr:\n\u{fffd}\r\x1b\nstderr_bytes: [{byte}, 13, 27]\nsource:\nsource\ncases:\n[{{}}]\ncode:\ncode",
                output.status,
            )
        );
    }
}

/// A child killed by a signal after printing the exact passing payload
/// crashed during teardown (#9067). That is reported but does not fail; a
/// wrong payload or a non-zero exit code still fails.
#[cfg(unix)]
#[test]
fn only_a_signal_after_the_exact_passing_payload_is_a_teardown_crash() {
    use std::os::unix::process::ExitStatusExt;

    let run = |stdout: &str, end: &str| {
        Command::new("node")
            .args([
                "-e",
                &format!("require('node:fs').writeSync(1, process.argv[1]); {end}"),
                stdout,
            ])
            .output()
            .unwrap()
    };
    let crash = "process.kill(process.pid, 'SIGSEGV')";
    let passed = run("{\"passed\":1}", crash);
    assert_eq!(passed.status.signal(), Some(11));
    let report = validate_runtime_output("vapor", "source", &json!([{}]), "code", &passed)
        .unwrap()
        .unwrap();
    assert_eq!(
        report.lines().take(4).collect::<Vec<_>>(),
        [
            "vapor: runtime child crashed after printing a passing result (#9067)",
            &format!("status: {}", passed.status),
            "exit_code: None",
            "signal: Some(11)",
        ]
    );
    for output in [
        run("{\"passed\":0}", crash),
        run("{\"passed\":1}", "process.exit(1)"),
    ] {
        let report =
            validate_runtime_output("vapor", "source", &json!([{}]), "code", &output).unwrap_err();
        assert_eq!(
            report.lines().next(),
            Some("vapor: runtime child failed"),
            "{report}"
        );
    }
}
