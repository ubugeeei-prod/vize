//! Original #6890 values: Vapor repair and a separately retained reference conflict.
#![expect(
    clippy::expect_used,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
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

const ORIGINAL: &str =
    include_str!("../../../tests/_fixtures/differential/jsx/child-values-6890-original.json");

fn compile(source: &str, mode: JsxOutputMode) -> Value {
    let allocator = Allocator::new();
    let result = compile_jsx(
        &allocator,
        source,
        JsxLang::Jsx,
        &JsxCompileConfig {
            default_mode: mode,
            ..Default::default()
        },
    );
    assert!(
        !result.has_errors(),
        "original producer diagnostics: {:?}",
        result.diagnostics
    );
    assert_eq!(result.components.len(), 1);
    json!({"code": result.module_code(), "functionName": "App"})
}

#[test]
fn original_child_values_match_vapor_with_an_explicit_vnode_frontier() {
    let original: Value = serde_json::from_str(ORIGINAL).expect("unchanged original ten phases");
    let source = original["source"].as_str().expect("original source");
    let packet = json!({
        "original": original,
        "vapor": compile(source, JsxOutputMode::Vapor),
        "vdom": compile(source, JsxOutputMode::Vdom),
    });
    if let Some(path) = std::env::var_os("VIZE_JSX_CHILD_MODULE_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec(&packet).expect("complete producer packet"),
        )
        .expect("capture modules before runtime execution");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut child = Command::new("node")
        .arg(root.join("tests/tooling/jsx-vapor-oracle/child-values.ts"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node and the pinned public providers are required");
    child
        .stdin
        .take()
        .expect("oracle stdin")
        .write_all(&serde_json::to_vec(&packet).expect("complete input"))
        .expect("send source modules");
    let output = child.wait_with_output().expect("runtime oracle result");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("one complete receipt");
    assert_eq!(receipt["originalPhases"], 10);
    assert_eq!(receipt["qualifiedVaporPhases"], 9);
    assert_eq!(receipt["referenceConflictPhases"], 1);
    assert_eq!(receipt["wholeCorpusQualified"], false);
    assert_eq!(receipt["issue6890Complete"], false);
}
