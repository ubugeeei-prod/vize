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
