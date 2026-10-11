//! Whole reported SFCs use the existing locked Vue runtime and VDOM oracle.

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

fn compile(source: &str) -> String {
    let descriptor = parse_sfc(source, Default::default()).expect("whole reported SFC parses");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            vapor: true,
            ..Default::default()
        },
        Default::default(),
        Default::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .expect("whole reported SFC compiles");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    assert!(result.css.is_none());
    assert!(result.macro_artifacts.is_empty());
    result.code.to_string()
}

pub fn compare(app: &str, child: &str, controls: &str) {
    let rows: Vec<Value> = serde_json::from_str(controls).expect("authored report controls");
    assert!(!rows.is_empty());
    for row in rows {
        let app = row["app"].as_str().unwrap_or(app);
        let child = row["child"].as_str().unwrap_or(child);
        let mut input = json!({"backend":"vapor", "appSource":app, "childSource":child,
            "code":compile(app), "child":compile(child), "context":row["context"],
            "steps":row["steps"]});
        for field in ["rootProps", "childModule"] {
            if let Some(value) = row.get(field) {
                input[field] = value.clone();
            }
        }
        let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/tooling/support/vapor-slot-defaults-7886.mjs");
        let mut process = Command::new("node")
            .arg(runner)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("locked real Vue runtime");
        process
            .stdin
            .take()
            .expect("runtime stdin")
            .write_all(&serde_json::to_vec(&input).expect("complete input"))
            .expect("whole sources reach runtime");
        let output = process.wait_with_output().expect("runtime exits");
        assert!(
            output.status.success(),
            "{}\nstdout={}\nstderr={}\napp={}\nchild={}",
            row["name"],
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
            input["code"],
            input["child"]
        );
        let trace: Value = serde_json::from_slice(&output.stdout).expect("complete runtime packet");
        assert_eq!(
            trace["current"], trace["oracle"],
            "{}: complete DOM, namespaces, events, updates and disposal",
            row["name"]
        );
        let current = trace["current"].as_array().expect("ordered phases");
        assert_eq!(
            current.len(),
            row["steps"].as_array().expect("ordered actions").len() + 2
        );
        assert!(
            !current[0]["tree"]
                .as_array()
                .expect("mounted tree")
                .is_empty()
        );
        assert_eq!(current.last().expect("unmount phase")["tree"], json!([]));
        if let Some(expected) = row.get("expectedNamespaces") {
            assert_eq!(&current[0]["namespaces"], expected, "{}", row["name"]);
        }
        if let Some(expected) = row.get("expectedTree") {
            assert_eq!(&current[0]["tree"], expected, "{}", row["name"]);
        }
        if let Some(expected) = row.get("expectedEvents") {
            assert_eq!(
                &current[current.len() - 2]["events"],
                expected,
                "{}",
                row["name"]
            );
        }
    }
}
