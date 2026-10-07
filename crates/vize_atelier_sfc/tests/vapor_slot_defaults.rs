//! #7886: complete original Vapor SFCs and independent reactive slot controls.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "regressions compare complete sources and independently authored observations"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::CodegenOptions;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcScriptOutputMode, compile_sfc_for_adapter, parse_sfc,
};

const APP: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-slot-defaults-7886/App.vue.txt"
);
const CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-slot-defaults-7886/Child.vue.txt"
);
const CONTROLS: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-slot-defaults-7886/controls.json"
);
const EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-slot-defaults-7886/runtime.expected.json"
);

fn compile(source: &str, vapor: bool) -> String {
    let descriptor = parse_sfc(source, Default::default()).expect("whole SFC parses");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            vapor,
            ..Default::default()
        },
        Default::default(),
        Default::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .expect("whole SFC compiles");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    assert!(result.css.is_none());
    assert!(result.macro_artifacts.is_empty());
    result.code.to_string()
}

fn trace(app: &str, child: &str, vapor: bool, context: Value, steps: Value) -> Value {
    let code = compile(app, vapor);
    let input = json!({ "backend": if vapor { "vapor" } else { "vdom" },
        "code": code, "child": compile(child, vapor), "context": context, "steps": steps });
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/vapor-sfc-runtime.mjs");
    let mut child = Command::new("node")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("locked real Vue runtime");
    child
        .stdin
        .take()
        .expect("runner stdin")
        .write_all(&serde_json::to_vec(&input).expect("whole input bytes"))
        .expect("whole source modules reach runtime");
    let output = child.wait_with_output().expect("runtime exits");
    assert!(
        output.status.success(),
        "status={}\nstdout={}\nstderr={}\n{code}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("complete runtime trace")
}

fn observations(trace: &Value) -> Value {
    Value::Array(
        trace
            .as_array()
            .expect("trace phases")
            .iter()
            .map(|phase| json!({"tree": phase["tree"], "events": phase["events"]}))
            .collect(),
    )
}

#[test]
fn complete_reported_vapor_sfcs_render_the_missing_slot_default() {
    let expected: Value = serde_json::from_str(EXPECTED).expect("authored expected result");
    let current = trace(APP, CHILD, true, json!({}), json!([]));
    assert_eq!(observations(&current), expected["original"]);
}

#[test]
fn slot_defaults_are_lazy_reactive_and_keep_nullish_value_semantics() {
    let rows: Vec<Value> = serde_json::from_str(CONTROLS).expect("independent controls");
    assert_eq!(rows.len(), 7);
    for row in rows {
        let app = row["app"].as_str().expect("whole App");
        let child = row["child"].as_str().expect("whole Child");
        let current = trace(
            app,
            child,
            true,
            row["context"].clone(),
            row["steps"].clone(),
        );
        let oracle = trace(
            app,
            child,
            false,
            row["context"].clone(),
            row["steps"].clone(),
        );
        assert_eq!(
            current, oracle,
            "{}: complete DOM, namespace, events and unmount",
            row["name"]
        );
        assert_eq!(observations(&current), row["expected"], "{}", row["name"]);
    }
}
