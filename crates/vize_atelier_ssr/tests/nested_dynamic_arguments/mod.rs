//! Whole original parent/child sources from the four observed SSR divergences.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "whole public compiler results and actual pinned Vue SSR evidence"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::CodegenOptions;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcCompileResult, SfcParseOptions,
    SfcScriptOutputMode, TemplateCompileOptions, compile_sfc_for_adapter, parse_sfc,
};
use vize_atelier_ssr::differential::{record_lanes, with_legacy_lane};

macro_rules! source {
    ($name:literal) => {
        include_str!(concat!(
            "../../../../tests/_fixtures/differential/compiler/ssr-nested-dynamic-arguments/",
            $name,
            ".vue.txt"
        ))
    };
}

const FILES: &[(&str, &str)] = &[
    ("Child", source!("Child")),
    ("Longhand", source!("Longhand")),
    ("Shorthand", source!("Shorthand")),
];

fn compile(name: &str, source: &str, map: bool, route: &str) -> SfcCompileResult {
    let filename = format!("{name}.vue");
    let parse = SfcParseOptions {
        filename: filename.as_str().into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("parse complete original SFC");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            parse,
            script: ScriptCompileOptions {
                id: Some(filename.as_str().into()),
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename.as_str().into()),
                ssr: true,
                is_prod: true,
                ..Default::default()
            },
            ..Default::default()
        },
        Default::default(),
        Default::default(),
        CodegenOptions {
            source_map: map,
            ..Default::default()
        },
        SfcScriptOutputMode::SeparateTemplate,
    )
    .expect("compile complete original SSR module");
    // Preserve both compiler routes' complete public results before validating
    // diagnostics, maps, or parity. These are compiler evidence, not Node runs.
    println!(
        "SSR_NESTED_COMPILER_PACKET={}",
        json!({ "name": name, "source": source, "mapEnabled": map,
            "route": route, "result": result })
    );
    assert!(result.errors.is_empty(), "{name}: {:?}", result.errors);
    assert!(result.warnings.is_empty(), "{name}: {:?}", result.warnings);
    assert!(result.css.is_none());
    assert!(result.macro_artifacts.is_empty());
    assert_eq!(result.map.is_some(), map, "{name}: map disposition");
    if let Some(map) = &result.map {
        assert_eq!(map["sources"], json!([filename]));
        assert_eq!(map["sourcesContent"], json!([source]));
    }
    result
}

fn selected(name: &str, source: &str, map: bool) -> SfcCompileResult {
    let (result, lanes) = record_lanes(|| compile(name, source, map, "selected"));
    assert_eq!(lanes, ["s4"], "{name}: actual selected SSR plan must emit");
    result
}

#[test]
fn original_nested_arguments_preserve_complete_selected_ssr_results_and_maps() {
    assert_eq!(
        FILES
            .iter()
            .map(|(_, source)| source.len())
            .collect::<Vec<_>>(),
        [1005, 347, 337]
    );
    for &(name, source) in FILES {
        let plain = selected(name, source, false);
        let mut mapped = selected(name, source, true);
        for map in [false, true] {
            let current = selected(name, source, map);
            let legacy = with_legacy_lane(|| compile(name, source, map, "explicit-legacy"));
            assert_eq!(
                serde_json::to_value(current).expect("complete selected public result"),
                serde_json::to_value(legacy).expect("complete legacy public result"),
                "{name}: every public field, map={map}"
            );
        }
        assert_eq!(plain.code, mapped.code, "{name}: additive source map");
        mapped.map = None;
        assert_eq!(
            serde_json::to_value(plain).expect("complete plain public result"),
            serde_json::to_value(mapped).expect("complete mapped public result"),
            "{name}: source map changes no other public field"
        );
    }
}

#[test]
fn quoted_arguments_keep_v_pre_and_element_identity_outcomes() {
    for source in [
        r#"<template><div v-pre v-bind:[keys['name]']].camel="literal"><Child is="vue:Child"/></div><p :id="active"/></template>"#,
        r#"<template><div v-pre><Child :[keys['name]']].camel="literal"/></div><p is="vue:Child"/></template>"#,
        r#"<template><table><tr is="vue:Row" :[keys['name]']].camel="value"/></table></template>"#,
    ] {
        let (current, lanes) = record_lanes(|| super::complete_v_pre_ssr(source));
        assert_eq!(lanes, ["s4"], "shared lower consumer: {source}");
        assert_eq!(
            current,
            with_legacy_lane(|| super::complete_v_pre_ssr(source)),
            "whole v-pre/element-identity result: {source}"
        );
    }
}

#[test]
fn whole_original_nested_argument_modules_render_like_pinned_vue() {
    let files: Vec<_> = FILES
        .iter()
        .map(|&(name, source)| {
            json!({ "name": name, "source": source,
            "current": selected(name, source, true),
            "legacy": with_legacy_lane(|| compile(name, source, true, "explicit-legacy")) })
        })
        .collect();
    let keys =
        json!({ "names]": ["first", "second"], "events]": ["update:first", "update:second"] });
    let input = json!({ "files": files, "states": [
        { "keys": keys, "indices": [0, 1], "index": 0, "value": "initial", "other": "untouched" },
        { "keys": keys, "indices": [0, 1], "index": 1, "value": "second", "other": "untouched" },
        { "keys": keys, "indices": [1, 0], "index": 0, "value": "reordered", "other": "other" }
    ] });
    let bytes = serde_json::to_vec(&input).expect("complete source-built module input");
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/nextest/{profile}/ssr-nested-dynamic-arguments"
    ));
    std::fs::create_dir_all(&directory).expect("evidence directory");
    let retain = |name: &str, bytes: &[u8]| {
        std::fs::write(directory.join(name), bytes)
            .expect("retain whole evidence before assertions");
    };
    retain("input.bin", &bytes);
    let git = Command::new("git")
        .args(["show", "--no-patch", "--format=%H%n%T%n%P", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("source/tree/parents");
    let source = json!({ "githubSha": std::env::var("GITHUB_SHA").ok(),
        "gitStatus": git.status.to_string(), "gitCode": git.status.code(),
        "gitStdout": git.stdout, "gitStderr": git.stderr });
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/ssr-nested-dynamic-arguments.mjs"),
        )
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("execute actual pinned Vue SSR helper");
    let pid = child.id();
    let write = child.stdin.take().expect("runtime stdin").write_all(&bytes);
    let output = child.wait_with_output().expect("reap actual runtime");
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
        &serde_json::to_vec(&receipt).expect("complete process receipt"),
    );
    // Full Check's original feature recipe preserves the entire packet in its
    // raw job log; the source-built modules and every runtime byte remain visible.
    println!(
        "SSR_NESTED_ARGUMENTS_PACKET={}",
        json!({ "input": input, "receipt": receipt,
        "stdout": output.stdout, "stderr": output.stderr })
    );
    assert_eq!(source["gitCode"], json!(0));
    assert!(write.is_ok(), "send full input: {write:?}");
    assert!(
        output.status.success(),
        "status: {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let observed: Value =
        serde_json::from_slice(&output.stdout).expect("complete actual runtime packet");
    assert_eq!(observed["stage"], json!("complete"));
    assert_eq!(
        observed["observations"]
            .as_array()
            .expect("whole HTML observations")
            .len(),
        12
    );
}
