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
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, VdomCompileOptions, compile_jsx};
use vize_l0::Allocator;

const CORPUS: &str = include_str!("vue_jsx_vapor_compat/fixtures/corpus.json");
const VERDICTS: &str = include_str!("vue_jsx_vapor_compat/fixtures/verdicts.json");

/// `source_map` selects the Relief VDOM codegen route instead of L2.
fn compile_case(case: &Value, mode: JsxOutputMode, source_map: bool) -> Value {
    let id = case["id"].as_str().expect("case id");
    let source = case["source"].as_str().expect("case source");
    let lang = match case["lang"].as_str() {
        None => JsxLang::Jsx,
        Some(lang) => {
            assert_eq!(lang, "tsx", "{id}: unknown corpus lang");
            JsxLang::Tsx
        }
    };
    let allocator = Allocator::new();
    let output = compile_jsx(
        &allocator,
        source,
        lang,
        &JsxCompileConfig {
            default_mode: mode,
            vdom: VdomCompileOptions {
                source_map,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    assert!(!output.has_errors(), "{id}: {:?}", output.diagnostics);
    assert_eq!(output.components.len(), 1, "{id}: one authored component");
    json!({
        "code": output.module_code(),
        "functionName": "App",
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
            let vapor = compile_case(case, JsxOutputMode::Vapor, false);
            let mut record = json!({
                "id": id,
                "source": source,
                "code": vapor["code"],
                "functionName": vapor["functionName"],
            });
            if case["babelCorpusId"].is_string() {
                record["vdom"] = compile_case(case, JsxOutputMode::Vdom, false);
                record["vdomSourceMapped"] = compile_case(case, JsxOutputMode::Vdom, true);
            }
            record
        })
        .collect::<Vec<_>>();

    let payload = json!({ "corpus": corpus, "compiled": compiled });
    if let Some(path) = std::env::var_os("VIZE_JSX_VAPOR_COMPAT_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec(&payload).expect("whole module packets"),
        )
        .expect("capture before runtime execution");
    }
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
        .write_all(&serde_json::to_vec(&payload).expect("complete oracle payload"))
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
    let verdicts: Value = serde_json::from_str(VERDICTS).expect("valid verdicts");
    let count = |verdict: &str| {
        verdicts["cases"]
            .as_object()
            .expect("verdict cases")
            .values()
            .filter(|row| row["verdict"] == verdict)
            .count()
    };
    let babel_cases = cases
        .iter()
        .filter(|case| case["babelCorpusId"].is_string())
        .count();
    assert_eq!(receipt["failures"], 0);
    assert_eq!(receipt["cases"], case_count);
    assert_eq!(receipt["equivalent"], count("equivalent"));
    assert_eq!(receipt["divergent"], count("divergent"));
    assert_eq!(receipt["babelCases"], babel_cases);
    assert_eq!(receipt["babelEquivalent"], babel_cases);
}
