//! Separate camel key evaluation/order witnesses; original cases stay intact.
#![expect(clippy::disallowed_macros, reason = "test-only complete receipt paths")]

use super::{record_lanes, v_pre_boundaries::compile, with_legacy_lane};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

const CORPUS: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/dynamic-component-camel/corpus.json"
);

#[test]
fn dynamic_component_camel_preserves_key_count_scope_falsy_and_prop_order() {
    let corpus: Value = serde_json::from_str(CORPUS).expect("finite source inventory");
    let cases = corpus["cases"].as_array().expect("all finite sources");
    assert_eq!(cases.len(), 12);
    let files: Vec<_> = cases
        .iter()
        .map(|case| {
            let name = case["name"].as_str().expect("case name");
            let source = case["source"].as_str().expect("whole authored source");
            let (current, lanes) = record_lanes(|| compile(name, source, "camel-selected"));
            let legacy = with_legacy_lane(|| compile(name, source, "camel-explicit-legacy"));
            println!(
                "SSR_CAMEL_COMPILER_PACKET={}",
                json!({
                    "case": case, "lanes": lanes, "current": current, "legacy": legacy
                })
            );
            assert_eq!(lanes, ["s4"], "{name}: real selected SSR route");
            assert_eq!(current, legacy, "{name}: all public fields/maps");
            json!({ "name": name, "source": source, "current": current, "legacy": legacy })
        })
        .collect();
    let input = json!({ "corpus": corpus, "files": files });
    let bytes = serde_json::to_vec(&input).expect("whole compiled input");
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/nextest/{profile}/ssr-component-camel-boundaries"
    ));
    std::fs::create_dir_all(&directory).expect("evidence directory");
    let retain = |name: &str, data: &[u8]| {
        std::fs::write(directory.join(name), data).expect("retain all bytes before assertions")
    };
    retain("input.bin", &bytes);
    let git = Command::new("git")
        .args(["show", "--no-patch", "--format=%H%n%T%n%P", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("observed checkout");
    let source = json!({"githubSha": std::env::var("GITHUB_SHA").ok(),
        "gitCode": git.status.code(), "gitStdout": git.stdout, "gitStderr": git.stderr});
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/ssr-component-camel-boundaries.mjs"),
        )
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("execute actual pinned compiler/runtime");
    let pid = child.id();
    let write = child.stdin.take().expect("whole input").write_all(&bytes);
    let output = child.wait_with_output().expect("completed child outcome");
    retain("stdout.bin", &output.stdout);
    retain("stderr.bin", &output.stderr);
    #[cfg(unix)]
    let signal = {
        use std::os::unix::process::ExitStatusExt;
        output.status.signal()
    };
    #[cfg(not(unix))]
    let signal: Option<i32> = None;
    let receipt = json!({"source": source, "pid": pid, "status": output.status.to_string(),
        "code": output.status.code(), "signal": signal, "success": output.status.success(),
        "stdinWriteError": write.as_ref().err().map(ToString::to_string),
        "inputBytes": bytes.len(), "stdoutBytes": output.stdout.len(), "stderrBytes": output.stderr.len()});
    retain(
        "receipt.json",
        &serde_json::to_vec(&receipt).expect("complete receipt"),
    );
    println!(
        "SSR_CAMEL_RUNTIME_PACKET={}",
        json!({"input":input,"receipt":receipt,"stdout":output.stdout,"stderr":output.stderr})
    );
    assert_eq!(source["gitCode"], json!(0));
    assert!(write.is_ok(), "{write:?}");
    assert!(
        output.status.success(),
        "status:{}\nstdout:{}\nstderr:{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let observed: Value = serde_json::from_slice(&output.stdout).expect("whole runtime JSON");
    assert_eq!(observed["stage"], json!("complete"));
    assert_eq!(
        observed["observations"]
            .as_array()
            .expect("whole renders")
            .len(),
        24
    );
}
