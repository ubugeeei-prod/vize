//! The dump feed must describe the product compile that actually ran.

#![expect(clippy::disallowed_macros, reason = "CLI assertions use format")]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use davinci_test_support::schema;
use vize_atelier_core::options::{
    CodegenExperimentalOptions, CodegenOptions, CustomElementMatcher, TemplateSyntaxMode,
};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture,
};
use vize_curator::inspector::{ProductCaptureSource, product_capture_value};
use vize_l0::{Allocator, dump::capture::StageCapture};

fn run(path: &Path, extra: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command.args(["dump", "--all-levels", "--json"]);
    command.args(extra);
    command.arg(path);
    command.output().unwrap()
}

#[test]
fn raw_template_uses_the_product_capture_constructor() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("template.html");
    let template = "<div :class=\"cls\">{{ msg }}</div>";
    fs::write(&path, template).unwrap();

    let output = run(&path, &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    let allocator = Allocator::default();
    let mut capture = StageCapture::new("dom");
    let _ = compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture(
        &allocator,
        template,
        DomCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        CodegenExperimentalOptions::default(),
        &mut capture,
    );
    let expected = product_capture_value(
        "vize-dump",
        ProductCaptureSource {
            path: Some(path.to_str().unwrap()),
            container: "raw-template",
            authored_syntax: "vue-template",
            compiled_syntax: "vue-template",
            template_span: None,
        },
        &capture,
    );
    assert_eq!(actual, expected);
    assert_eq!(actual["schema_version"], 2);
    assert_eq!(actual["outcome"]["kind"], "accepted");
    assert_eq!(actual["target"], "dom");
    assert!(
        actual["pages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|page| page["level"] == "l4")
    );
    assert!(
        !actual["pages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|page| page["level"] == "l3")
    );
    let schema_json: serde_json::Value = serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join("docs/davinci/plan/product-stage-feed.schema.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(schema::validate(&schema_json, &actual, "$"), Ok(()));
}

#[test]
fn sfc_input_has_authored_template_span_and_no_unrun_dom_l3() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("App.vue");
    let source = "<script>export default { name: 'App' }</script>\n<template><p>hi</p></template>";
    fs::write(&path, source).unwrap();
    let output = run(&path, &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["source"]["container"], "vue-sfc");
    assert_eq!(value["source"]["authored_syntax"], "vue-template");
    assert_eq!(value["source"]["compiled_syntax"], "vue-template");
    let start = value["source"]["template_span"]["start"].as_u64().unwrap() as usize;
    let end = value["source"]["template_span"]["end"].as_u64().unwrap() as usize;
    assert_eq!(&source[start..end], "<p>hi</p>");
    assert_eq!(value["target"], "dom");
    assert!(
        !value["pages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|page| page["level"] == "l3")
    );
}

#[test]
fn pug_input_identifies_the_derived_template() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("template.pug");
    fs::write(&path, "p hello").unwrap();
    let output = run(&path, &[]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["source"]["authored_syntax"], "pug");
    assert_eq!(value["source"]["compiled_syntax"], "vue-template");
    assert!(value["source"]["template_span"].is_null());
    assert_eq!(value["pages"][0]["text"], "<p>hello</p>");
}

#[test]
fn unsupported_named_pass_is_rejected_instead_of_running_a_noop() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("template.html");
    fs::write(&path, "<div>hi</div>").unwrap();
    let output = run(&path, &["--pipeline", "l2(template-complexity)"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("invalid value")
    );
}

#[test]
fn page_directory_and_unobserved_channels_are_explicit() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("template.html");
    fs::write(&path, "<div>hi</div>").unwrap();
    let dir = temp.path().join("pages");
    let timing = temp.path().join("timing.json");
    let remarks = temp.path().join("remarks.json");
    let output = run(
        &path,
        &[
            "--dump-dir",
            dir.to_str().unwrap(),
            "--dump-after-change",
            "--timing-json",
            timing.to_str().unwrap(),
            "--remarks",
            remarks.to_str().unwrap(),
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let feed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let written: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("product-stage-feed.json")).unwrap()).unwrap();
    assert_eq!(feed, written);
    assert!(
        fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.path().extension().is_some_and(|ext| ext == "dump"))
    );
    let timing_value: serde_json::Value =
        serde_json::from_slice(&fs::read(timing).unwrap()).unwrap();
    let remarks_value: serde_json::Value =
        serde_json::from_slice(&fs::read(remarks).unwrap()).unwrap();
    assert_eq!(timing_value["observed"], false);
    assert_eq!(remarks_value["observed"], false);
    assert!(timing_value["timings"].as_array().unwrap().is_empty());
    assert!(remarks_value["remarks"].as_array().unwrap().is_empty());
}
