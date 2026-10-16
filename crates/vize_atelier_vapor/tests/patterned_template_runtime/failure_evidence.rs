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

/// The rendered payload can be right and the process can still be unusable.
/// SIGSEGV after `{"passed":1}` failed the v0.453.0 gate (#9067); ignoring
/// that signal is what let the crash through.
#[cfg(unix)]
#[test]
fn a_signal_after_the_exact_passing_payload_fails() {
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
    let report =
        validate_runtime_output("vapor", "source", &json!([{}]), "code", &passed).unwrap_err();
    assert_eq!(
        report.lines().take(4).collect::<Vec<_>>(),
        [
            "vapor: runtime child failed",
            &format!("status: {}", passed.status),
            "exit_code: None",
            "signal: Some(11)",
        ]
    );
    let needle = "{\"passed\":1}";
    assert!(
        report.split_once(needle).is_some(),
        "rendered result must stay in the evidence:\n{report}"
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

/// Both real bundler producers and the addon-free observer must exit naturally.
/// A teardown signal fails this the way the original gate failed.
#[test]
fn mounted_runtime_producers_and_observer_exit_naturally() {
    use std::path::Path;

    let child = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/mounted-runtime-native-exit.mjs");
    let output = Command::new("node").arg(&child).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "producer and observer teardown must exit 0\nstatus: {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status,
    );
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(
            output.status.signal(),
            None,
            "native teardown signaled {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            output.status,
        );
    }
    assert_eq!(output.stdout, br#"{"passed":1}"#);
    assert!(
        output.stderr.is_empty(),
        "unexpected teardown stderr:\n{stderr}"
    );
}
