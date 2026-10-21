//! Mounted semantic differential against the real `vue-jsx-vapor` package.
#![expect(
    clippy::expect_used,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the integration oracle executes installed Node tooling"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx};
use vize_l0::Allocator;

const CORPUS: &str = include_str!("vue_jsx_vapor_compat/fixtures/corpus.json");

fn compile_case(id: &str, source: &str, mode: JsxOutputMode, function_name: &str) -> Value {
    let allocator = Allocator::new();
    let output = compile_jsx(
        &allocator,
        source,
        JsxLang::Jsx,
        &JsxCompileConfig {
            default_mode: mode,
            ..Default::default()
        },
    );
    assert!(!output.has_errors(), "{id}: {:?}", output.diagnostics);
    assert_eq!(output.components.len(), 1, "{id}: one authored component");
    json!({
        "code": output.module_code(),
        "functionName": function_name,
    })
}

#[test]
fn real_vue_jsx_vapor_mount_and_updates_match_inventory() {
    let corpus: Value = serde_json::from_str(CORPUS).expect("valid semantic corpus");
    let cases = corpus["cases"].as_array().expect("corpus cases");
    let case_count = cases.len();
    let compiled = cases
        .iter()
        .map(|case| {
            let id = case["id"].as_str().expect("case id");
            let source = case["source"].as_str().expect("case source");
            let vapor = compile_case(id, source, JsxOutputMode::Vapor, "render");
            let mut record = json!({
                "id": id,
                "source": source,
                "code": vapor["code"],
                "functionName": vapor["functionName"],
            });
            if case["babelCorpusId"].is_string() {
                record["vdom"] = compile_case(id, source, JsxOutputMode::Vdom, "App");
            }
            record
        })
        .collect::<Vec<_>>();

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut child = Command::new("node")
        .arg(root.join("tests/tooling/jsx-vapor-oracle/oracle.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node and the pinned vue-jsx-vapor oracle are required");
    child
        .stdin
        .take()
        .expect("oracle stdin")
        .write_all(
            &serde_json::to_vec(&json!({ "corpus": corpus, "compiled": compiled }))
                .expect("complete oracle payload"),
        )
        .expect("send Vize output to the oracle");
    let output = child
        .wait_with_output()
        .expect("real runtime oracle process");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let receipt: Value = serde_json::from_slice(&output.stdout).expect("oracle receipt");
    assert_eq!(receipt["failures"], 0);
    assert_eq!(receipt["cases"], case_count);
    assert_eq!(receipt["equivalent"], case_count - 1);
    assert_eq!(receipt["divergent"], 1);
    assert_eq!(receipt["babelCases"], 4);
    assert_eq!(receipt["babelEquivalent"], 4);
}
