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
use vize_atelier_sfc::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    compile_sfc_for_adapter_with_stage_capture, parse_sfc,
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
    assert_eq!(
        output.stderr,
        b"error: invalid value 'l2(template-complexity)' for '--pipeline <PIPELINE>'\n  [possible values: dom, ssr, vapor]\n\nFor more information, try '--help'.\n"
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

#[test]
fn reused_directory_removes_only_unchanged_pages_owned_by_the_prior_feed() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.html");
    let script_only = temp.path().join("Script.vue");
    let dir = temp.path().join("pages");
    fs::write(&template, "<div>hi</div>").unwrap();
    fs::write(&script_only, "<script>export default {}</script>").unwrap();
    let dump_dir = ["--dump-dir", dir.to_str().unwrap()];
    let first = run(&template, &dump_dir);
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let generated: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "dump"))
        .collect();
    assert!(!generated.is_empty());
    fs::write(dir.join("notes.dump"), "user notes").unwrap();
    fs::write(dir.join("keep.txt"), "user file").unwrap();

    let second = run(&script_only, &dump_dir);
    assert_eq!(second.status.code(), Some(0), "{second:?}");
    let feed: serde_json::Value = serde_json::from_slice(&second.stdout).unwrap();
    assert_eq!(feed["outcome"]["kind"], "unavailable");
    assert!(feed["pages"].as_array().unwrap().is_empty());
    for path in generated {
        assert!(!path.exists(), "stale page: {}", path.display());
    }
    assert_eq!(fs::read(dir.join("notes.dump")).unwrap(), b"user notes");
    assert_eq!(fs::read(dir.join("keep.txt")).unwrap(), b"user file");
    let written: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("product-stage-feed.json")).unwrap()).unwrap();
    assert_eq!(written, feed);
}

#[test]
fn modified_prior_page_is_preserved_and_blocks_reuse() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.html");
    let dir = temp.path().join("pages");
    fs::write(&template, "<div>hi</div>").unwrap();
    let dump_dir = ["--dump-dir", dir.to_str().unwrap()];
    let first = run(&template, &dump_dir);
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let page = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|ext| ext == "dump"))
        .unwrap();
    let old_feed = fs::read(dir.join("product-stage-feed.json")).unwrap();
    fs::write(&page, "user edits").unwrap();
    let second = run(&template, &dump_dir);
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(
        second.stderr,
        format!(
            "dump: {}: refusing to remove modified {}\n",
            template.display(),
            page.display()
        )
        .as_bytes()
    );
    assert_eq!(fs::read(&page).unwrap(), b"user edits");
    assert_eq!(
        fs::read(dir.join("product-stage-feed.json")).unwrap(),
        old_feed
    );
}

#[test]
fn scoped_sfc_capture_matches_filename_configured_product_compile() {
    let temp = tempfile::tempdir().unwrap();
    let source = "<template><p>hi</p></template><style scoped>p { color: red }</style>";
    let mut outputs = Vec::new();
    for name in ["One.vue", "Two.vue"] {
        let path = temp.path().join(name);
        fs::write(&path, source).unwrap();
        let output = run(&path, &[]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let filename = path.to_str().unwrap();
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: filename.into(),
                ..SfcParseOptions::default()
            },
        )
        .unwrap();
        let mut options = SfcCompileOptions::default();
        options.parse.filename = filename.into();
        let (compiled, capture) = compile_sfc_for_adapter_with_stage_capture(
            &descriptor,
            options,
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
            SfcScriptOutputMode::SeparateTemplate,
            SfcCompileExperimentalOptions::default(),
        )
        .unwrap();
        let expected = product_capture_value(
            "vize-dump",
            ProductCaptureSource {
                path: Some(filename),
                container: "vue-sfc",
                authored_syntax: "vue-template",
                compiled_syntax: "vue-template",
                template_span: Some(vize_l0::Span::new(
                    source.find("<p>").unwrap() as u32,
                    (source.find("<p>").unwrap() + "<p>hi</p>".len()) as u32,
                )),
            },
            &capture,
        );
        assert_eq!(actual, expected);
        // Scoped CSS is assembled by the SFC host after the L4 render page.
        outputs.push(compiled.css.unwrap());
    }
    assert_ne!(
        outputs[0], outputs[1],
        "product scoped CSS ignored source filename"
    );
}

#[test]
fn external_template_source_is_rejected_before_capture() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("External.vue");
    fs::write(&path, "<template src=\"./template.html\"/>").unwrap();
    let output = run(&path, &[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        format!(
            "dump: {}: external <template src> is not supported by native dump\n",
            path.display()
        )
        .as_bytes()
    );
}
