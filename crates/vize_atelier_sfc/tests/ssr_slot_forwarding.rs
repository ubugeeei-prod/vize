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
const KEYED_FIXTURE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/ssr-keyed-template-components.json"
);
const TARGETS: &[&str] = &["dom", "ssr", "vapor", "vapor-ssr"];

fn keyed_sources() -> Vec<(String, Vec<(String, String)>)> {
    let fixture: Value = serde_json::from_str(KEYED_FIXTURE).expect("whole keyed fixture");
    fixture["cases"]
        .as_array()
        .expect("all five source vectors")
        .iter()
        .map(|case| {
            let mut files = ["Value", "TriggerPush", "TriggerVNode"]
                .into_iter()
                .map(|name| {
                    (
                        name.to_owned(),
                        fixture["files"][name]
                            .as_str()
                            .expect("whole common SFC")
                            .to_owned(),
                    )
                })
                .collect::<Vec<_>>();
            files.push((
                "App".to_owned(),
                case["source"].as_str().expect("whole case SFC").to_owned(),
            ));
            (case["name"].as_str().expect("case name").to_owned(), files)
        })
        .collect()
}

fn artifact_directory() -> std::path::PathBuf {
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let directory =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../target/nextest/{profile}"));
    std::fs::create_dir_all(&directory).expect("existing test artifact directory");
    directory
}

fn compile(name: &str, source: &str, target: &str, map: bool) -> SfcCompileResult {
    compile_in_build(name, source, target, map, true, false)
}

fn compile_in_build(
    name: &str,
    source: &str,
    target: &str,
    map: bool,
    is_prod: bool,
    comments: bool,
) -> SfcCompileResult {
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
                is_prod,
                compiler_options: comments.then(|| vize_atelier_dom::DomCompilerOptions {
                    comments,
                    ..Default::default()
                }),
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
    assert!(
        result.warnings.is_empty(),
        "{target}/{name}: {:?}",
        result.warnings
    );
    assert!(result.css.is_none());
    assert!(result.macro_artifacts.is_empty());
    if let Some(map) = &result.map {
        assert_eq!(map["sources"], json!([filename]));
    }
    result
}

#[test]
fn conditional_ssr_vnodes_hydrate_and_update_like_each_official_compiler_build() {
    let fixture = include_str!(
        "../../../tests/_fixtures/differential/compiler/ssr-conditional-vnode-comments.json"
    );
    let source: Value = serde_json::from_str(fixture).expect("complete source fixture");
    let directory = artifact_directory();
    for (build, is_prod) in [("development", false), ("production", true)] {
        let files: Vec<_> = ["Inner", "App"]
            .into_iter()
            .map(|name| {
                let source = source["files"][name].as_str().expect("whole SFC");
                let current = compile_in_build(name, source, "ssr", true, is_prod, true);
                assert!(current.map.is_some(), "{build}/{name}: real native map");
                json!({ "name": name, "source": source, "current": current })
            })
            .collect();
        let input = json!({ "conditionalComments": { "fixture": fixture, "build": build, "files": files } });
        let prefix = format!("ssr-conditional-vnode-comments-{build}");
        std::fs::write(
            directory.join(format!("{prefix}-input.json")),
            serde_json::to_vec(&input).expect("complete native input"),
        )
        .expect("retain native modules before runtime assertions");
        let mut child = Command::new("node")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/tooling/support/ssr-slot-forwarding.mjs"),
            )
            .env("NODE_ENV", build)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("same compiler build and Vue runtime");
        child
            .stdin
            .take()
            .expect("oracle stdin")
            .write_all(&serde_json::to_vec(&input).expect("whole modules"))
            .expect("send native modules");
        let output = child.wait_with_output().expect("runtime exits");
        std::fs::write(
            directory.join(format!("{prefix}-stdout.json")),
            &output.stdout,
        )
        .expect("retain whole stdout");
        std::fs::write(
            directory.join(format!("{prefix}-stderr.txt")),
            &output.stderr,
        )
        .expect("retain whole stderr");
        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let observed: Value = serde_json::from_slice(&output.stdout).expect("whole observations");
        assert_eq!(
            observed["conditionalComments"]["observations"]
                .as_array()
                .expect("all paired rows")
                .len(),
            4
        );
        assert_eq!(observed["conditionalComments"]["stage"], "complete");
    }
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
fn keyed_component_loop_vectors_keep_every_codegen_field_and_map_additive() {
    for target in TARGETS {
        for (case, files) in keyed_sources() {
            for (name, source) in files {
                let plain = compile(&name, &source, target, false);
                let mut mapped = compile(&name, &source, target, true);
                assert!(mapped.map.is_some(), "{target}/{case}/{name}: real map");
                assert_eq!(plain.code, mapped.code, "{target}/{case}/{name}");
                mapped.map = None;
                assert_eq!(
                    serde_json::to_value(plain).expect("whole result"),
                    serde_json::to_value(mapped).expect("whole mapped result"),
                    "{target}/{case}/{name}: every public field except the map"
                );
            }
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
    let keyed: Vec<_> = keyed_sources()
        .into_iter()
        .flat_map(|(name, files)| {
            TARGETS.iter().map(move |target| {
                let files: Vec<_> = files
                    .iter()
                    .map(|(name, source)| {
                        json!({ "name": name, "source": source,
                            "current": compile(name, source, target, true) })
                    })
                    .collect();
                json!({ "name": name, "target": target, "files": files })
            })
        })
        .collect();
    let input = json!({ "cases": cases,
        "keyedTemplates": { "fixture": KEYED_FIXTURE, "cases": keyed } });
    let directory = artifact_directory();
    std::fs::write(
        directory.join("ssr-keyed-template-components-native-input.json"),
        serde_json::to_vec(&input).expect("whole native compilation packet"),
    )
    .expect("retain native packet before runtime assertions");
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
    std::fs::write(
        directory.join("ssr-keyed-template-components-runtime.stdout.json"),
        &output.stdout,
    )
    .expect("retain complete raw runtime stdout on success or failure");
    std::fs::write(
        directory.join("ssr-keyed-template-components-runtime.stderr.txt"),
        &output.stderr,
    )
    .expect("retain complete raw runtime stderr on success or failure");
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
    assert_eq!(observations["keyedTemplates"]["stage"], "complete");
    assert_eq!(
        observations["keyedTemplates"]["observations"]
            .as_array()
            .expect("all five vectors, three providers, two slot branches, three values")
            .len(),
        90
    );
    std::fs::write(
        directory.join("ssr-slot-forwarding-runtime.json"),
        serde_json::to_vec(&json!({ "input": input, "observations": observations }))
            .expect("serialize complete artifact"),
    )
    .expect("retain source, output and real runtime evidence");
}
