//! Artifact transport for three existing unit projections, never an oracle.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use serde_json::json;
use sha2::{Digest, Sha256};

fn hex(bytes: &[u8]) -> std::string::String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest(bytes: &[u8]) -> std::string::String {
    hex(Sha256::digest(bytes).as_slice())
}

fn binary_digest(path: &Path) -> io::Result<std::string::String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(hex(digest.finalize().as_slice()))
}

fn variable(name: &str) -> io::Result<std::string::String> {
    std::env::var(name).map_err(io::Error::other)
}

fn save(script: &str, template: &str, code: &str) -> io::Result<()> {
    let profile = variable("NEXTEST_PROFILE")?;
    if profile != "full" {
        return Err(io::Error::other(
            "unit capture requires the full TC profile",
        ));
    }
    let head = variable("GITHUB_SHA")?;
    let run = variable("GITHUB_RUN_ID")?;
    let attempt = variable("GITHUB_RUN_ATTEMPT")?;
    if head.len() != 40
        || !head.bytes().all(|byte| byte.is_ascii_hexdigit())
        || run.is_empty()
        || !run.bytes().all(|byte| byte.is_ascii_digit())
        || attempt.is_empty()
        || !attempt.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(io::Error::other("invalid unit capture source/run identity"));
    }
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/virtual_ts/loose_is_listener_tests.rs");
    let compiled_unit = include_bytes!("../loose_is_listener_tests.rs");
    let current_unit = fs::read(&source)?;
    if compiled_unit.as_slice() != current_unit.as_slice() {
        return Err(io::Error::other(
            "compiled unit source differs from checkout",
        ));
    }
    let capture_source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/virtual_ts/loose_is_listener_tests/current_capture.rs");
    let compiled_capture = include_bytes!("current_capture.rs");
    let current_capture = fs::read(capture_source)?;
    if compiled_capture.as_slice() != current_capture.as_slice() {
        return Err(io::Error::other("compiled capture differs from checkout"));
    }
    let binary = fs::canonicalize(std::env::current_exe()?)?;
    let input = json!({
        "script": script,
        "template": template,
        "scriptOffset": 0,
        "templateOffset": 0,
        "options": "VirtualTsOptions::default()",
        "entry": "generate_virtual_ts_with_offsets",
    });
    let input_sha256 = digest(&serde_json::to_vec(&input).map_err(io::Error::other)?);
    let packet = json!({
        "schema": "vize.canon.current-unit-projection.v1",
        "qualification": false,
        "source": {
            "head": head,
            "compiledHead": option_env!("GITHUB_SHA"),
            "runId": run,
            "runAttempt": attempt,
            "unitPath": "crates/vize_canon/src/virtual_ts/loose_is_listener_tests.rs",
            "unitSha256": digest(compiled_unit),
            "captureSha256": digest(compiled_capture),
        },
        "binary": {
            "path": binary,
            "bytes": fs::metadata(&binary)?.len(),
            "sha256": binary_digest(&binary)?,
        },
        "input": input,
        "inputSha256": input_sha256,
        "code": code,
        "codeBytes": code.len(),
        "codeSha256": digest(code.as_bytes()),
    });
    // This directory already enters the existing Rust cohort artifact.
    let directory = PathBuf::from(variable("GITHUB_WORKSPACE")?)
        .join("target/nextest")
        .join(profile)
        .join("canon-unit-projections")
        .join(format!("{run}-{attempt}-{head}"));
    fs::create_dir_all(&directory)?;
    // Retries retain separate records rather than replacing an earlier packet.
    let output = directory.join(format!("{input_sha256}-{}.json", std::process::id()));
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    serde_json::to_writer_pretty(&mut output, &packet).map_err(io::Error::other)?;
    output.write_all(b"\n")
}

pub(super) fn record(script: &str, template: &str, code: &str) {
    if std::env::var("COHORT").ok().as_deref() != Some("typechecker") {
        return;
    }
    if let Err(error) = save(script, template, code) {
        // Transport never accepts or changes the existing test's status.
        eprintln!("canon unit projection capture unavailable: {error}");
    }
}
