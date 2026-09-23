//! The Node runner that mounts both compilers' output under the published
//! runtime and records its state and identity trace.

use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use serde_json::Value;

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
