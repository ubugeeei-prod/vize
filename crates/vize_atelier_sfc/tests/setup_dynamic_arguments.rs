//! #7881: original dynamic setup argument modules, maps and actual mounted Vue.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "whole original source/public result/runtime regression evidence"
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

macro_rules! source {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/compiler/setup-dynamic-args-7881/",
            $name,
            ".vue.txt"
        ))
    };
}
const FILES: &[(&str, &str)] = &[
    ("Child", source!("Child")),
    ("App", source!("App")),
    ("Const", source!("Const")),
    ("Maybe", source!("Maybe")),
    ("Mutable", source!("Mutable")),
    ("Names", source!("Names")),
    ("For", source!("For")),
    ("Slot", source!("Slot")),
    ("Static", source!("Static")),
];
const MODES: &[(&str, SfcScriptOutputMode)] = &[
    ("inline", SfcScriptOutputMode::InlineTemplate),
    ("separate", SfcScriptOutputMode::SeparateTemplate),
];

fn compile(name: &str, source: &str, mode: SfcScriptOutputMode, map: bool) -> SfcCompileResult {
    let filename = format!("{name}.vue");
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.as_str().into(),
            ..Default::default()
        },
    )
    .expect("parse original whole SFC");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            parse: SfcParseOptions {
                filename: filename.as_str().into(),
                ..Default::default()
            },
            script: ScriptCompileOptions {
                id: Some(filename.as_str().into()),
                ..Default::default()
            },
            template: TemplateCompileOptions {
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
        mode,
    )
    .expect("compile complete original module");
    assert!(result.errors.is_empty(), "{name}: {:?}", result.errors);
    assert!(result.warnings.is_empty(), "{name}: {:?}", result.warnings);
    assert!(result.css.is_none());
    assert!(result.macro_artifacts.is_empty());
    assert_eq!(result.map.is_some(), map);
    if let Some(map) = &result.map {
        assert_eq!(map["sources"], json!([filename]));
        assert_eq!(map["sourcesContent"], json!([source]));
    }
    result
}

#[test]
fn original_setup_argument_maps_preserve_complete_public_results() {
    for &(name, source) in FILES {
        for &(_, mode) in MODES {
            let plain = compile(name, source, mode, false);
            let mut mapped = compile(name, source, mode, true);
            assert_eq!(plain.code, mapped.code, "{name}: additive map");
            mapped.map = None;
            assert_eq!(
                serde_json::to_value(plain).expect("whole plain result"),
                serde_json::to_value(mapped).expect("whole mapped result"),
                "{name}: all public fields"
            );
        }
    }
}

#[test]
fn original_setup_ref_arguments_use_the_same_bindings_as_their_values() {
    for &(_, mode) in MODES {
        let result = compile("App", source!("App"), mode, true);
        assert_eq!(
            serde_json::to_value(result.bindings).expect("complete actual original setup facts"),
            json!({ "isScriptSetup": true, "propsAliases": {}, "bindings": {
                "ref": "setup-maybe-ref", "evt": "setup-ref", "attr": "setup-ref", "n": "setup-ref"
            } })
        );
    }
}

#[test]
fn whole_original_setup_argument_modules_mount_like_official_vue() {
    let modes: Vec<_> = MODES
        .iter()
        .map(|&(label, mode)| {
            let files: Vec<_> = FILES.iter().map(|&(name, source)| {
            json!({ "name": name, "source": source, "current": compile(name, source, mode, true) })
        }).collect();
            json!({ "mode": label, "files": files })
        })
        .collect();
    let input = json!({ "modes": modes });
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../target/nextest/{profile}"));
    std::fs::create_dir_all(&directory).expect("existing evidence directory");
    let git = Command::new("git")
        .args(["show", "--no-patch", "--format=%H%n%T%n%P", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("literal source/tree/parents");
    let source = json!({ "githubSha": std::env::var("GITHUB_SHA").ok(),
        "gitStatus": git.status.to_string(), "gitCode": git.status.code(),
        "gitStdout": git.stdout, "gitStderr": git.stderr });
    let retain = |name: &str, bytes: &[u8]| {
        std::fs::write(
            directory.join(format!("setup-dynamic-arguments-{name}")),
            bytes,
        )
        .expect("retain full original outcome before assertions");
    };
    retain(
        "input.json",
        &serde_json::to_vec(&json!({ "input": &input, "source": &source }))
            .expect("serialize original inputs/complete current results/source"),
    );
    assert_eq!(source["gitCode"], json!(0));
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/setup-dynamic-arguments-7881.mjs"),
        )
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("execute pinned official Vue DOM runtime");
    let write = child
        .stdin
        .take()
        .expect("runtime stdin")
        .write_all(input.to_string().as_bytes());
    let output = child
        .wait_with_output()
        .expect("reap runtime before asserting write or exit");
    retain("stdout.json", &output.stdout);
    retain("stderr.txt", &output.stderr);
    retain("process.json", &serde_json::to_vec(&json!({ "source": &source,
        "status": output.status.to_string(), "code": output.status.code(),
        "success": output.status.success(), "stdinWriteError": write.as_ref().err().map(ToString::to_string) }))
        .expect("serialize actual process outcome"));
    assert!(write.is_ok(), "send full input: {write:?}");
    assert!(
        output.status.success(),
        "status: {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let observed: Value = serde_json::from_slice(&output.stdout).expect("complete runtime packet");
    assert_eq!(
        observed["observations"]
            .as_array()
            .expect("all32mounts")
            .len(),
        32
    );
    retain(
        "runtime.json",
        &serde_json::to_vec(&json!({ "input": input, "referenceAndRuntime": observed,
        "source": source, "nativeLevelCredit": false }))
        .expect("whole current/reference/runtime packet"),
    );
}
