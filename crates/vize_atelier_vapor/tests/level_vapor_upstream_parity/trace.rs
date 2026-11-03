//! The Node runner that mounts both compilers' output under the published
//! runtime and records its state and identity trace.

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
pub(super) struct Fixture {
    pub(super) source: String,
    pub(super) context: Value,
    pub(super) steps: Vec<Value>,
    pub(super) expected: Vec<Value>,
}

pub(super) fn trace(runner: &str, input: Value) -> Vec<Value> {
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

pub(super) fn assert_native_upstream_trace(
    source: &str,
    context: Value,
    steps: Value,
    expected: Vec<Value>,
) {
    assert_native_upstream_trace_with_targets(source, context, steps, expected, json!([]));
}

pub(super) fn assert_native_upstream_trace_with_targets(
    source: &str,
    context: Value,
    steps: Value,
    expected: Vec<Value>,
    external_targets: Value,
) {
    let allocator = Allocator::new();
    let before = WalkCounts::snapshot();
    let compiled = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
    );
    assert!(
        compiled.error_messages.is_empty(),
        "{source}: {:?}",
        compiled.error_messages
    );
    assert_eq!(
        WalkCounts::snapshot().since(before).total_walks(),
        0,
        "{source}: TS-33 must exercise native L3"
    );
    let vize = trace(
        "davinci-mounted-trace.mjs",
        json!({
            "backend": "vapor",
            "code": compiled.code,
            "context": context.clone(),
            "steps": steps.clone(),
            "identities": true,
            "externalTargets": external_targets.clone(),
        }),
    );
    let upstream = trace(
        "davinci-upstream-vapor-trace.mjs",
        json!({"source": source, "context": context, "steps": steps, "externalTargets": external_targets}),
    );
    assert_eq!(vize, expected, "{source}: Vize native L3 trace");
    assert_eq!(
        upstream, expected,
        "{source}: official compiler-vapor trace"
    );
    assert_eq!(vize, upstream, "{source}: TS-33 behavior-level parity");
}
