//! Separate source-bound whole v-pre witnesses; original populations stay intact.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "complete public compiler results and strict actual runtime custody"
)]

use super::{
    CodegenOptions, CustomElementMatcher, ScriptCompileOptions, SfcCompileExperimentalOptions,
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, StyleCompileOptions,
    TemplateCompileOptions, TemplateSyntaxMode, compile_sfc_for_adapter_with_experimental_options,
    parse_sfc, record_lanes, with_legacy_lane,
};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

const CORPUS: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/v-pre-literal-boundary/corpus.json"
);
macro_rules! source {
    ($name:literal) => {
        (
            $name,
            include_str!(concat!(
                "../../../tests/_fixtures/differential/compiler/v-pre-literal-boundary/",
                $name,
                ".vue.txt"
            )),
        )
    };
}
const FILES: &[(&str, &str)] = &[
    source!("original-1"),
    source!("original-2"),
    source!("original-3"),
    source!("order-1"),
    source!("order-2"),
    source!("order-3"),
    source!("order-4"),
    source!("literal-other"),
];

pub(super) fn compile(name: &str, source: &str, route: &str) -> Value {
    let filename: vize_l0::String = "format-v-pre-content.vue".into();
    let parse = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("parse whole source");
    let result = compile_sfc_for_adapter_with_experimental_options(
        &descriptor,
        SfcCompileOptions {
            parse,
            script: ScriptCompileOptions {
                id: Some(filename.clone()),
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename),
                ssr: true,
                ..Default::default()
            },
            style: StyleCompileOptions::default(),
            vapor: false,
            scope_id: None,
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions {
            source_map: true,
            prefix_identifiers: true,
            ..Default::default()
        },
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    );
    // Keep complete failures in the raw log as well as every successful public
    // result. No failed source is replaced with a recovered-empty module.
    println!(
        "SSR_V_PRE_COMPILER_OUTCOME name={name:?} route={route:?} source={source:?} result={result:?}"
    );
    let result = result.expect("compile whole v-pre module");
    let result = serde_json::to_value(result).expect("all public result fields");
    println!(
        "SSR_V_PRE_COMPILER_PACKET={}",
        json!({ "name": name, "route": route, "source": source, "result": result })
    );
    assert_eq!(result["errors"], json!([]));
    assert_eq!(result["warnings"], json!([]));
    result
}

#[test]
fn literal_v_pre_whole_modules_render_like_pinned_vue() {
    let corpus: Value = serde_json::from_str(CORPUS).expect("source custody inventory");
    assert_eq!(
        corpus["cases"]
            .as_array()
            .expect("whole source inventory")
            .len(),
        8
    );
    let files: Vec<_> = FILES
        .iter()
        .map(|&(name, source)| {
            let (current, lanes) = record_lanes(|| compile(name, source, "selected"));
            assert_eq!(lanes, ["s4"], "{name}: actual typed SSR plan");
            let legacy = with_legacy_lane(|| compile(name, source, "explicit-legacy"));
            assert_eq!(current, legacy, "{name}: all public compiler fields");
            json!({ "name": name, "source": source, "current": current, "legacy": legacy })
        })
        .collect();
    let input = json!({ "files": files, "state": {
        "active": "active-id", "keys": { "name]": "data-key" }, "value": "normal-value"
    } });
    observe_runtime(
        input,
        "../../tests/tooling/support/ssr-v-pre-boundaries.mjs",
        "ssr-v-pre-boundaries",
        "SSR_V_PRE_RUNTIME_PACKET",
        16,
    );
}

pub(super) fn observe_runtime(
    input: Value,
    helper: &str,
    directory_name: &str,
    packet_prefix: &str,
    expected_count: usize,
) {
    let bytes = serde_json::to_vec(&input).expect("whole actual compiled modules");
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/nextest/{profile}/{directory_name}"));
    std::fs::create_dir_all(&directory).expect("evidence directory");
    let retain = |name: &str, data: &[u8]| {
        std::fs::write(directory.join(name), data).expect("retain all bytes before asserting")
    };
    retain("input.bin", &bytes);
    let git = Command::new("git")
        .args(["show", "--no-patch", "--format=%H%n%T%n%P", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("observed source checkout");
    let source = json!({ "githubSha": std::env::var("GITHUB_SHA").ok(),
        "gitStatus": git.status.to_string(), "gitCode": git.status.code(),
        "gitStdout": git.stdout, "gitStderr": git.stderr });
    let mut child = Command::new("node")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(helper))
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("execute actual pinned compiler/runtime");
    let pid = child.id();
    let write = child
        .stdin
        .take()
        .expect("complete stdin")
        .write_all(&bytes);
    let output = child.wait_with_output().expect("actual child outcome");
    retain("stdout.bin", &output.stdout);
    retain("stderr.bin", &output.stderr);
    #[cfg(unix)]
    let signal = {
        use std::os::unix::process::ExitStatusExt;
        output.status.signal()
    };
    #[cfg(not(unix))]
    let signal: Option<i32> = None;
    let receipt = json!({ "source": source, "pid": pid, "status": output.status.to_string(),
        "code": output.status.code(), "signal": signal, "success": output.status.success(),
        "stdinWriteError": write.as_ref().err().map(ToString::to_string),
        "inputBytes": bytes.len(), "stdoutBytes": output.stdout.len(), "stderrBytes": output.stderr.len() });
    retain(
        "receipt.json",
        &serde_json::to_vec(&receipt).expect("complete receipt"),
    );
    println!(
        "{packet_prefix}={}",
        json!({ "input": input, "receipt": receipt,
        "stdout": output.stdout, "stderr": output.stderr })
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
    let observed: Value =
        serde_json::from_slice(&output.stdout).expect("whole observed runtime JSON");
    assert_eq!(observed["stage"], json!("complete"));
    assert_eq!(
        observed["observations"]
            .as_array()
            .expect("whole renders")
            .len(),
        expected_count
    );
}
