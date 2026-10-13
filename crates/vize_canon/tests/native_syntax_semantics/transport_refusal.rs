//! Instrumented denial delegates ordinary CLI calls to the real native runtime.

use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde_json::{Value, json};

const MESSAGE: &str = "native diagnostic transport intentionally unavailable";

pub(super) fn without_transport(root: &Path, native: &Path) -> PathBuf {
    // Match the actual runtime's async JSON-RPC authority. An arbitrary shell
    // filename would instead select the legacy msgpack transport.
    fs::create_dir_all(root.join(".bin")).unwrap();
    let wrapper = root.join(".bin/native-without-diagnostic-transport");
    let attempts = root.join(".bin/denied-transports.log");
    let peer = root.join(".bin/startup-denial.ts");
    let captures = root.join(".bin/denied-bootstraps");
    fs::write(&attempts, "").unwrap();
    fs::write(&peer, include_str!("startup_denial.ts")).unwrap();
    fs::create_dir(&captures).unwrap();
    let quoted = native.to_string_lossy().replace('\'', "'\"'\"'");
    let quoted_attempts = attempts.to_string_lossy().replace('\'', "'\"'\"'");
    let quoted_peer = peer.to_string_lossy().replace('\'', "'\"'\"'");
    let quoted_captures = captures.to_string_lossy().replace('\'', "'\"'\"'");
    // Consume the actual bootstrap before denying it. Immediate process exit
    // races the writer against the reader and does not qualify one refusal.
    fs::write(&wrapper, format!("#!/bin/sh\ncase \"$1\" in --api|--lsp) printf '%s\\n' \"$1\" >> '{quoted_attempts}'; exec node '{quoted_peer}' '{quoted_captures}' \"$@\";; esac\nexec '{quoted}' \"$@\"\n")).unwrap();
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
    wrapper
}

pub(super) fn capture_and_replay(root: &Path, wrapper: &Path) -> Value {
    // Capture the original native-client invocations before replay adds files.
    let mut originals: Vec<_> = fs::read_dir(root.join(".bin/denied-bootstraps"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    originals.sort();
    let packets: Vec<_> = originals
        .into_iter()
        .map(|path| {
            let original: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            let argv: Vec<_> = original["argv"]
                .as_array()
                .unwrap()
                .iter()
                .map(|arg| arg.as_str().unwrap())
                .collect();
            let frame: Vec<u8> =
                serde_json::from_value(original["wholeRequestFrame"].clone()).unwrap();
            let mut process = Command::new(wrapper)
                .args(&argv)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let replay_pid = process.id();
            process.stdin.take().unwrap().write_all(&frame).unwrap();
            let replay = process.wait_with_output().unwrap();
            let replay_packet: Value = serde_json::from_slice(
                &fs::read(
                    root.join(".bin/denied-bootstraps")
                        .join(format!("{replay_pid}.json")),
                )
                .unwrap(),
            )
            .unwrap();
            json!({"wholeOriginalNativeClientBootstrap":original,
            "directCapturedBootstrapReplay":{
                "argv":argv,"exitCode":replay.status.code(),
                "wholeStdout":replay.stdout,"wholeStderr":replay.stderr,
                "wholeReplayBootstrapPacket":replay_packet,
            }})
        })
        .collect();
    json!(packets)
}

pub(super) fn assert_exact_denial(packets: &Value, attempts: &str) {
    let packets = packets.as_array().unwrap();
    assert!(!packets.is_empty(), "actual bootstrap denial is required");
    // PID filenames do not establish invocation order. Bind the count and
    // entire typed trace records, so no attempted transport loses custody.
    let mut traced: Vec<_> = attempts.split_inclusive('\n').map(str::to_owned).collect();
    let mut captured: Vec<_> = packets
        .iter()
        .map(|packet| {
            format!(
                "{}\n",
                packet["wholeOriginalNativeClientBootstrap"]["argv"]
                    .as_array()
                    .unwrap()
                    .first()
                    .unwrap()
                    .as_str()
                    .unwrap()
            )
        })
        .collect();
    traced.sort();
    captured.sort();
    assert_eq!(traced, captured, "every attempted transport needs a packet");
    for packet in packets {
        let original = &packet["wholeOriginalNativeClientBootstrap"];
        let request = frame_body(&original["wholeRequestFrame"]);
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "initialize");
        assert!(request["id"].is_number() || request["id"].is_string());
        assert!(matches!(
            original["argv"]
                .as_array()
                .unwrap()
                .first()
                .unwrap()
                .as_str(),
            Some("--api" | "--lsp")
        ));
        assert_eq!(
            frame_body(&original["wholeResponseFrame"]),
            json!({"jsonrpc":"2.0","id":request["id"],
                "error":{"code":-32041,"message":MESSAGE}})
        );
        let replay = &packet["directCapturedBootstrapReplay"];
        assert_eq!(replay["exitCode"], 41);
        assert_eq!(replay["argv"], original["argv"]);
        assert_eq!(replay["wholeStdout"], original["wholeResponseFrame"]);
        assert_eq!(
            replay["wholeStderr"],
            json!(format!("{MESSAGE}\n").as_bytes())
        );
        assert_eq!(
            replay["wholeReplayBootstrapPacket"]["wholeRequestFrame"],
            original["wholeRequestFrame"]
        );
        assert_eq!(
            replay["wholeReplayBootstrapPacket"]["wholeResponseFrame"],
            original["wholeResponseFrame"]
        );
        assert_eq!(
            replay["wholeReplayBootstrapPacket"]["runtime"],
            original["runtime"]
        );
        assert_eq!(
            original["runtime"]["wholePeerSource"],
            include_str!("startup_denial.ts")
        );
    }
}

fn frame_body(frame: &Value) -> Value {
    let frame: Vec<u8> = serde_json::from_value(frame.clone()).unwrap();
    let header_end = frame
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .unwrap()
        + 4;
    let (header, payload) = frame.split_at_checked(header_end).unwrap();
    let header = std::str::from_utf8(header).unwrap();
    let length: usize = header
        .trim()
        .strip_prefix("Content-Length: ")
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(payload.len(), length);
    serde_json::from_slice(payload).unwrap()
}
