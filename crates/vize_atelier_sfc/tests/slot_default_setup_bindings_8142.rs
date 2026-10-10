//! Whole original typed slot callback inputs, compiler results and runtime laws.
#![cfg(feature = "legacy-dom-differential")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "complete independently authored compiler and runtime packet custody"
)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    StyleCompileOptions, TemplateCompileOptions, compile_sfc_for_adapter, parse_sfc,
};
use vize_l0::profiler::global_profiler;

const CASES: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/slot-default-setup-bindings-8142/original-runtime-cases.json"
);

#[test]
fn inline_setup_defaults_preserve_all_original_typed_slot_runtime_laws() {
    let corpus: Value = serde_json::from_str(CASES).expect("whole independently authored inputs");
    let cases = corpus["cases"].as_array().expect("six original inputs");
    assert_eq!(cases.len(), 6);
    let mut observations = Vec::new();
    for case in cases {
        let filename = case["filename"].as_str().expect("filename");
        let source = case["source"].as_str().expect("whole source");
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: filename.into(),
                ..Default::default()
            },
        )
        .expect("authored complete SFC");
        let mut compilers = Vec::new();
        for legacy in [false, true] {
            let profiler = global_profiler();
            profiler.clear();
            profiler.enable();
            let compile = || {
                compile_sfc_for_adapter(
                    &descriptor,
                    SfcCompileOptions {
                        parse: SfcParseOptions {
                            filename: filename.into(),
                            ..Default::default()
                        },
                        script: ScriptCompileOptions {
                            id: Some(filename.into()),
                            ..Default::default()
                        },
                        template: TemplateCompileOptions {
                            id: Some(filename.into()),
                            compiler_options: Some(vize_atelier_dom::DomCompilerOptions::default()),
                            ..Default::default()
                        },
                        style: StyleCompileOptions {
                            id: filename.into(),
                            ..Default::default()
                        },
                        vapor: false,
                        scope_id: None,
                    },
                    TemplateSyntaxMode::Standard,
                    CustomElementMatcher::default(),
                    CodegenOptions::default(),
                    SfcScriptOutputMode::InlineTemplate,
                )
            };
            let result = if legacy {
                vize_atelier_dom::differential::with_legacy_lane(compile)
            } else {
                compile()
            };
            let counters = profiler.counter_summary();
            profiler.disable();
            profiler.clear();
            compilers.push(json!({
                "lane": if legacy { "legacy" } else { "selected" },
                "completeCompileResult": result,
                "completeCounterDebug": format!("{counters:?}"),
                "counterEntries": counters.entries.iter().map(|entry| json!({
                    "name": entry.name, "total": entry.total
                })).collect::<Vec<_>>(),
            }));
        }
        observations.push(json!({ "id": case["id"], "filename": filename,
            "source": source, "completeDescriptorDebug": format!("{descriptor:?}"),
            "compilers": compilers }));
    }
    let executable = std::env::current_exe().expect("actual compiled producer");
    let native_bytes = std::fs::read(&executable).expect("authentic producer bytes");
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/slot-default-setup-bindings-8142");
    std::fs::create_dir_all(&target).expect("complete packet retention directory");
    let output = json!({
        "schema": "vize.slot-default-setup-bindings.actual-whole-sfc-packets", "version": 1,
        "producer": { "executable": executable, "sha256": digest(&native_bytes),
            "pid": std::process::id(), "githubSha": std::env::var("GITHUB_SHA").ok(),
            "argv": std::env::args().collect::<Vec<_>>() },
        "authoredCases": corpus, "authoredCasesSha256": digest(CASES.as_bytes()),
        "runtimeReceipt": target.join("runtime-packets.json"), "observations": observations,
    });
    std::fs::write(
        target.join("compiler-packets.json"),
        serde_json::to_vec_pretty(&output).expect("complete raw compiler packets"),
    )
    .expect("retain all raw compiler results before runtime or assertions");
    run_runtime(&output, &target);
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn run_runtime(input: &Value, target: &std::path::Path) {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/slot-default-setup-bindings-runtime.mjs");
    let bytes = serde_json::to_vec(input).expect("whole actual compiler input");
    let mut child = Command::new("node")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("actual pinned runtime child");
    let pid = child.id();
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(&bytes)
        .expect("whole original inputs and actual modules");
    let result = child
        .wait_with_output()
        .expect("actual runtime process return");
    std::fs::write(target.join("runtime.stdout.json"), &result.stdout)
        .expect("retain whole stdout before judging status");
    std::fs::write(target.join("runtime.stderr.txt"), &result.stderr)
        .expect("retain whole stderr before judging status");
    std::fs::write(
        target.join("runtime-child-process.json"),
        serde_json::to_vec_pretty(&json!({ "pid": pid,
            "status": format!("{}", result.status), "code": result.status.code(),
            "inputSha256": digest(&bytes) }))
        .expect("complete runtime child custody"),
    )
    .expect("retain actual child return before assertions");
    assert!(
        result.status.success(),
        "runtime {pid} status={}\nstdout={}\nstderr={}",
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let receipt: Value = serde_json::from_slice(&result.stdout).expect("complete runtime receipt");
    assert_eq!(receipt["node"]["pid"], pid);
    assert_eq!(receipt["input"]["packets"], *input);
}
