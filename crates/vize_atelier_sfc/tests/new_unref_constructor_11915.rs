//! #11915: imported constructors must receive `new` after inline unwrapping.

use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, compile_sfc, compile_sfc_for_adapter,
    parse_sfc,
};
use vize_l0::{String, cstr};

#[test]
fn inline_unref_constructors_execute_like_vue_with_ordinary_call_controls() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/new-unref-constructor-11915/cases.json"
    ))
    .unwrap();
    let mut captured = Vec::new();
    for case in cases.as_array().unwrap() {
        let source = cstr!(
            "<script setup>\n{}\n</script>\n<template><Cam :look-at=\"{}\" /></template>\n",
            case["script"].as_str().unwrap(),
            case["expression"].as_str().unwrap()
        );
        let descriptor = parse_sfc(
            &source,
            SfcParseOptions {
                filename: "Cam.vue".into(),
                ..Default::default()
            },
        )
        .expect("original report and controls parse");
        let mut options = SfcCompileOptions::default();
        options.parse.filename = "Cam.vue".into();
        options.template.is_prod = true;
        let result = compile_sfc_for_adapter(
            &descriptor,
            options.clone(),
            TemplateSyntaxMode::Standard,
            Default::default(),
            Default::default(),
            SfcScriptOutputMode::InlineTemplate,
        )
        .expect("complete production inline component");
        assert_eq!(
            result.errors.len(),
            0,
            "{}: {:?}",
            case["id"],
            result.errors
        );
        let mapped = compile_sfc_for_adapter(
            &descriptor,
            options,
            TemplateSyntaxMode::Standard,
            Default::default(),
            CodegenOptions {
                source_map: true,
                ..Default::default()
            },
            SfcScriptOutputMode::InlineTemplate,
        )
        .expect("mapped production inline component");
        assert_eq!(mapped.code, result.code, "mapped bytes: {}", case["id"]);
        let map = mapped.map.as_ref().expect("complete inline source map");
        assert_eq!(map["sourcesContent"], json!([source]), "{}", case["id"]);
        captured.push(json!({"id":case["id"], "source":source, "result":result, "mapped":mapped}));
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut child = Command::new("node")
        .arg(root.join("tests/tooling/new-unref-constructor-runtime.mjs"))
        .current_dir(&root)
        .env("NODE_ENV", "production")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run actual Vue constructor consumer");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&captured).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert_eq!(
        output.status.code(),
        Some(0),
        "complete modules: {}\nstdout: {}\nstderr: {}",
        serde_json::to_string(&captured).unwrap(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn separate_template_ssr_constructor_control_keeps_its_complete_module() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/new-unref-constructor-11915/Layout.vue.txt"
    );
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).unwrap();
    let mut options = SfcCompileOptions {
        parse,
        ..Default::default()
    };
    options.template.ssr = true;
    let result = compile_sfc(&descriptor, options).unwrap();
    assert_eq!(result.errors.len(), 0);
    assert_eq!(
        result.code.as_str(),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/new-unref-constructor-11915/module.expected.txt"
        )
    );
}
