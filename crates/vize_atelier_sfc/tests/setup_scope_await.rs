//! #7972: complete setup-scope await modules and actual SSR context restoration.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    reason = "complete public compiler results cross the independent Vue runtime oracle"
)]

use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions,
    compile_sfc_with_template_syntax_and_codegen_options, parse_sfc,
};

const REPORTED: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/setup-scope-await/Probe.vue.txt");
const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/setup-scope-await/cases.json");

#[test]
fn whole_setup_scope_await_modules_restore_instance_injection_and_ssr_lifecycle() {
    let mut fixtures = vec![json!({"name": "reported", "source": REPORTED})];
    fixtures.extend(serde_json::from_str::<Vec<Value>>(CASES).unwrap());
    assert_eq!(fixtures.len(), 15);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".to_owned());
    let capture = root
        .join("target/nextest")
        .join(profile)
        .join("setup-scope-await");
    fs::create_dir_all(&capture).unwrap();
    let mut observations = Vec::new();
    for fixture in &fixtures {
        let name = fixture["name"].as_str().unwrap();
        let source = fixture["source"].as_str().unwrap();
        let filename = format!("{name}.vue");
        let parse = SfcParseOptions {
            filename: filename.as_str().into(),
            ..Default::default()
        };
        let descriptor = parse_sfc(source, parse.clone()).unwrap();
        for ssr in [false, true] {
            let compile = |source_map| {
                compile_sfc_with_template_syntax_and_codegen_options(
                    &descriptor,
                    SfcCompileOptions {
                        parse: parse.clone(),
                        template: TemplateCompileOptions {
                            ssr,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    TemplateSyntaxMode::Standard,
                    CodegenOptions {
                        source_map,
                        filename: filename.as_str().into(),
                        ..Default::default()
                    },
                )
                .unwrap()
            };
            let (off, on) = (compile(false), compile(true));
            let mut off = serde_json::to_value(off).unwrap();
            let on = serde_json::to_value(on).unwrap();
            assert!(off["map"].is_null());
            assert!(on["map"].is_object());
            off["map"] = on["map"].clone();
            assert_eq!(off, on, "maps are additive for {name}, ssr={ssr}");
            observations.push(json!({
                "name": name, "source": source, "filename": filename,
                "target": if ssr { "ssr" } else { "dom" }, "current": on,
            }));
        }
    }
    let input = serde_json::to_vec(&json!({
        "capture": capture, "cases": observations,
    }))
    .unwrap();
    fs::write(capture.join("input.json"), &input).unwrap();
    let mut child = Command::new("node")
        .arg(root.join("tests/tooling/support/setup-scope-await.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&input).unwrap();
    let output = child.wait_with_output().unwrap();
    fs::write(capture.join("stdout.json"), &output.stdout).unwrap();
    fs::write(capture.join("stderr.txt"), &output.stderr).unwrap();
    fs::write(
        capture.join("process.json"),
        serde_json::to_vec(&json!({
            "success": output.status.success(), "exitCode": output.status.code(),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(receipt["complete"], true);
    assert_eq!(receipt["observations"].as_array().unwrap().len(), 30);
}
