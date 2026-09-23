//! TS-33: S3 Vapor and official compiler-vapor output must have the same
//! mounted behavior under the same published runtime, including node identity.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use serde::Deserialize;
use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    source: String,
    context: Value,
    steps: Vec<Value>,
    expected: Vec<Value>,
}

fn trace(runner: &str, input: Value) -> Vec<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support")
        .join(runner);
    let mut child = Command::new("node")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start TS-33 mounted runner (install workspace JS dependencies first)");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{runner} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
}

#[test]
fn s3_branch_matches_official_vapor_state_and_identity_trace() {
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/davinci-ts33-vapor-branch.json"
    ))
    .unwrap();
    assert!(!fixture.steps.is_empty());
    assert_eq!(fixture.expected.len(), fixture.steps.len() + 2);

    let allocator = Allocator::new();
    let before = WalkCounts::snapshot();
    let compiled = compile_vapor(
        &allocator,
        &fixture.source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
    );
    assert!(
        compiled.error_messages.is_empty(),
        "{:?}",
        compiled.error_messages
    );
    assert_eq!(
        WalkCounts::snapshot().since(before).total_walks(),
        0,
        "TS-33 must exercise native S3"
    );

    let vize = trace(
        "davinci-mounted-trace.mjs",
        json!({
            "backend": "vapor",
            "code": compiled.code,
            "context": fixture.context,
            "steps": fixture.steps,
            "identities": true,
        }),
    );
    let upstream = trace(
        "davinci-upstream-vapor-trace.mjs",
        json!({
            "source": fixture.source,
            "context": fixture.context,
            "steps": fixture.steps,
        }),
    );
    assert_eq!(vize, fixture.expected, "Vize native S3 trace");
    assert_eq!(upstream, fixture.expected, "official compiler-vapor trace");
    assert_eq!(vize, upstream, "TS-33 behavior-level parity");
}
