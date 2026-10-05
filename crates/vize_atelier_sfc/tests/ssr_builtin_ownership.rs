//! #7891: complete original default components and real built-in SSR semantics.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "whole source/code/map/runtime regression evidence"
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
            "../../../tests/_fixtures/differential/compiler/ssr-builtins-7891/",
            $name,
            ".vue.txt"
        ))
    };
}
const FILES: &[(&str, &str)] = &[
    ("Async", source!("Async")),
    ("App", source!("App")),
    ("Explicit", source!("Explicit")),
    ("FallbackOnly", source!("FallbackOnly")),
    ("OtherSlot", source!("OtherSlot")),
    ("DynamicSlot", source!("DynamicSlot")),
    ("NoTag", source!("NoTag")),
    ("DynamicTag", source!("DynamicTag")),
    ("Attrs", source!("Attrs")),
    ("DefaultOnly", source!("DefaultOnly")),
];

fn compile(name: &str, source: &str, map: bool) -> SfcCompileResult {
    let filename = format!("{name}.vue");
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.as_str().into(),
            ..Default::default()
        },
    )
    .expect("parse whole original SFC");
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
    .expect("compile complete SSR module");
    assert!(result.errors.is_empty(), "{name}: {:?}", result.errors);
    assert!(result.warnings.is_empty(), "{name}: {:?}", result.warnings);
    assert!(result.css.is_none());
    assert!(result.macro_artifacts.is_empty());
    if let Some(map) = &result.map {
        assert_eq!(map["sources"], json!([filename]));
    }
    result
}

#[test]
fn original_builtin_sfc_maps_preserve_all_public_fields_and_code_bytes() {
    for &(name, source) in FILES {
        let plain = compile(name, source, false);
        let mut mapped = compile(name, source, true);
        assert_eq!(plain.code, mapped.code, "{name}: additive map");
        mapped.map = None;
        assert_eq!(
            serde_json::to_value(plain).expect("whole plain result"),
            serde_json::to_value(mapped).expect("whole mapped result"),
            "{name}: all public fields"
        );
    }
}

#[test]
fn whole_original_builtins_render_like_official_complete_sfc_defaults() {
    let files: Vec<_> = FILES.iter().map(|&(name, source)| {
        json!({ "name": name, "source": source, "current": compile(name, source, true) })
    }).collect();
    let input = json!({ "files": files });
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/ssr-builtins-7891.mjs"),
        )
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("execute real official SSR runtime");
    child
        .stdin
        .take()
        .expect("runtime stdin")
        .write_all(input.to_string().as_bytes())
        .expect("send every whole generated result");
    let output = child.wait_with_output().expect("runtime exits");
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
            .expect("all original and control cases")
            .len(),
        18
    );
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../target/nextest/{profile}"));
    std::fs::create_dir_all(&directory).expect("existing evidence directory");
    std::fs::write(
        directory.join("ssr-builtin-ownership-runtime.json"),
        serde_json::to_vec(&json!({ "input": input, "referenceAndRuntime": observed,
            "sourceHead": std::env::var("GITHUB_SHA").ok(), "nativeLevelCredit": false }))
        .expect("serialize full code/map/runtime evidence"),
    )
    .expect("retain complete original inputs, current modules and official reference graphs");
}
