//! Whole raw stdio corpus for bounded TypeScript content-mapper frames (#3984).
#![expect(clippy::unwrap_used, reason = "tests assert by panicking")]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use vize_l0::{String, cstr};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    name: String,
    input: String,
    input_sha256: String,
    stdout: String,
    stdout_sha256: String,
    status: i32,
    stderr: String,
}

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/content-mapper-frame-bounds")
}

fn pinned_bytes(file: &str, expected_hash: &str) -> Vec<u8> {
    let bytes = std::fs::read(corpus().join(file)).unwrap();
    let mut actual_hash = String::with_capacity(64);
    for byte in Sha256::digest(&bytes) {
        actual_hash.push_str(cstr!("{byte:02x}").as_str());
    }
    assert_eq!(actual_hash, expected_hash, "{file}");
    bytes
}

#[test]
fn raw_content_mapper_stdio_preserves_complete_responses_and_fatal_frames() {
    let cases: Vec<Case> =
        serde_json::from_slice(&std::fs::read(corpus().join("cases.json")).unwrap()).unwrap();
    assert_eq!(cases.len(), 14, "retain the whole authored framing corpus");
    for case in cases {
        let input = pinned_bytes(&case.input, &case.input_sha256);
        let expected_stdout = pinned_bytes(&case.stdout, &case.stdout_sha256);
        // Exercise three write chunk sizes; pipe writes can coalesce. Reader
        // laws separately force internal fragmentation and concatenation.
        for chunk_size in [1, 7, 1024] {
            let mut child = Command::new(env!("CARGO_BIN_EXE_vize"))
                .arg("content-mapper")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let mut stdin = child.stdin.take().unwrap();
            for chunk in input.chunks(chunk_size) {
                if let Err(error) = stdin.write_all(chunk) {
                    assert_eq!(
                        error.kind(),
                        std::io::ErrorKind::BrokenPipe,
                        "{}",
                        case.name
                    );
                    break; // A rejected header can close before its unused body.
                }
            }
            drop(stdin);
            let output = child.wait_with_output().unwrap();
            assert_eq!(
                output.status.code(),
                Some(case.status),
                "{} chunk={chunk_size}",
                case.name
            );
            assert_eq!(
                output.stdout, expected_stdout,
                "{} chunk={chunk_size}",
                case.name
            );
            assert_eq!(
                output.stderr,
                case.stderr.as_bytes(),
                "{} chunk={chunk_size}",
                case.name
            );
        }
    }
}
