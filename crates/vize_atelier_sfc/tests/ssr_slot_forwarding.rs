//! #7890: preserve the reporter's whole SFC and execute both real SSR slot branches.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "complete original compiler and production runtime observations"
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
            "../../../tests/_fixtures/differential/compiler/ssr-slot-forwarding/",
            $name,
            ".vue.txt"
        ))
    };
}
const FILES: &[(&str, &str)] = &[
    ("Forward", source!("Forward")),
    ("Inner", source!("Inner")),
    ("InnerNamed", source!("InnerNamed")),
    ("App", source!("App")),
];

fn compile(name: &str, source: &str, target: &str, map: bool) -> SfcCompileResult {
    let filename = format!("{name}.vue");
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.as_str().into(),
            ..Default::default()
        },
    )
    .expect("parse complete original SFC");
    let ssr = target.ends_with("ssr");
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
                ssr,
                is_prod: true,
                ..Default::default()
            },
            vapor: target.starts_with("vapor"),
            ..Default::default()
        },
        Default::default(),
        Default::default(),
        CodegenOptions {
            source_map: map,
            ..Default::default()
        },
        if target == "vapor" {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
    )
    .expect("compile complete module");
    assert!(
        result.errors.is_empty(),
        "{target}/{name}: {:?}",
        result.errors
    );
    assert_eq!(result.warnings.len(), usize::from(target == "vapor-ssr"));
    if target == "vapor-ssr" {
        assert_eq!(
            result
                .warnings
                .first()
                .expect("documented fallback")
                .code
                .as_deref(),
            Some("VAPOR_SSR_FALLBACK")
        );
    }
    assert!(result.css.is_none());
    assert!(result.macro_artifacts.is_empty());
    if let Some(map) = &result.map {
        assert_eq!(map["sources"], json!([filename]));
    }
    result
}

#[test]
fn original_forwarding_sfc_keeps_maps_additive_and_complete_public_outputs() {
    for target in ["dom", "ssr", "vapor", "vapor-ssr"] {
        for &(name, source) in FILES {
            let plain = compile(name, source, target, false);
            let mut mapped = compile(name, source, target, true);
            assert_eq!(plain.code, mapped.code, "{target}/{name}");
            mapped.map = None;
            assert_eq!(
                serde_json::to_value(plain).expect("whole result"),
                serde_json::to_value(mapped).expect("whole mapped result"),
                "{target}/{name}: every public field except the requested map"
            );
        }
    }
}

#[test]
fn original_forwarding_defaults_render_like_whole_official_sfc_components() {
    let cases: Vec<_> = ["dom", "ssr", "vapor", "vapor-ssr"]
        .into_iter()
        .map(|target| {
            let files: Vec<_> = FILES
                .iter()
                .map(|&(name, source)| {
                    json!({ "name": name, "source": source,
                    "current": compile(name, source, target, true) })
                })
                .collect();
            json!({ "target": target, "files": files })
        })
        .collect();
    let input = json!({ "cases": cases });
    let mut child = Command::new("node")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/tooling/support/ssr-slot-forwarding.mjs"),
        )
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run independent whole-component oracle");
    child
        .stdin
        .take()
        .expect("oracle stdin")
        .write_all(input.to_string().as_bytes())
        .expect("send complete original modules");
    let output = child
        .wait_with_output()
        .expect("whole-component oracle exits");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let observations: Value =
        serde_json::from_slice(&output.stdout).expect("complete reference and actual observations");
    assert_eq!(
        observations["observations"]
            .as_array()
            .expect("all cases")
            .len(),
        12
    );
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../target/nextest/{profile}"));
    std::fs::create_dir_all(&directory).expect("existing test artifact directory");
    std::fs::write(
        directory.join("ssr-slot-forwarding-runtime.json"),
        serde_json::to_vec(&json!({ "input": input, "observations": observations }))
            .expect("serialize complete artifact"),
    )
    .expect("retain source, output and real runtime evidence");
}
