//! RFC 823 shape matching through real DOM, Vapor, and SSR runtimes.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_options};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn compile(backend: &str, source: &str) -> String {
    let allocator = Allocator::new();
    match backend {
        "vdom" => {
            let (_, errors, result) = compile_template_with_options(
                &allocator,
                source,
                DomCompilerOptions {
                    prefix_identifiers: true,
                    experimental_patterned_template: true,
                    ..Default::default()
                },
            );
            assert!(errors.is_empty(), "DOM errors: {errors:?}");
            format!("{}\n{}", result.preamble, result.code)
        }
        "ssr" => {
            let (_, errors, result) = compile_ssr_with_options(
                &allocator,
                source,
                SsrCompilerOptions {
                    experimental_patterned_template: true,
                    ..Default::default()
                },
            );
            assert!(errors.is_empty(), "SSR errors: {errors:?}");
            format!("{}\n{}", result.preamble, result.code)
        }
        "vapor" => {
            let result = compile_vapor(
                &allocator,
                source,
                VaporCompilerOptions {
                    prefix_identifiers: true,
                    experimental_patterned_template: true,
                    ..Default::default()
                },
            );
            assert!(
                result.error_messages.is_empty(),
                "Vapor errors: {:?}",
                result.error_messages
            );
            result.code.to_string()
        }
        _ => panic!("unknown backend: {backend}"),
    }
}

fn check(source: &str, cases: Value) {
    for backend in ["vdom", "vapor", "ssr"] {
        let code = compile(backend, source);
        let mut child = Command::new("node")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/tooling/support/patterned-template-runtime.mjs"),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start real runtime runner");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(
                json!({"backend": backend, "code": code, "cases": cases})
                    .to_string()
                    .as_bytes(),
            )
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{backend}: {}\ncode:\n{code}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            json!({"passed": cases.as_array().unwrap().len()})
        );
    }
}

fn paragraph(text: &str) -> Value {
    json!([{"tag": "p", "attributes": {}, "children": [text]}])
}

#[test]
fn object_patterns_require_property_presence_without_reading_wildcards() {
    check(
        r#"<template v-match="subject"><p v-when="{ field: _ }">present</p><p v-when="_">absent</p></template>"#,
        json!([
            {"context": {"subject": {}}, "trees": [paragraph("absent")]},
            {"scenario": "own-undefined", "trees": [paragraph("present")]},
            {"scenario": "inherited-undefined", "trees": [paragraph("present")]},
            {"scenario": "inherited-getter", "trees": [paragraph("present")], "reads": 0},
            {"context": {"subject": null}, "trees": [paragraph("absent")]},
            {"context": {"subject": false}, "trees": [paragraph("absent")]}
        ]),
    );
}

#[test]
fn object_patterns_box_primitives_and_check_nested_keys() {
    check(
        r#"<template v-match="subject"><p v-when="{ length: 3 }">three</p><p v-when="{ nested: { field: _ } }">nested</p><p v-when="_">other</p></template>"#,
        json!([
            {"context": {"subject": "abc"}, "trees": [paragraph("three")]},
            {"context": {"subject": {"nested": {}}}, "trees": [paragraph("other")]},
            {"context": {"subject": {"nested": {"field": null}}}, "trees": [paragraph("nested")]}
        ]),
    );
}

#[test]
fn array_patterns_distinguish_empty_exact_and_rest_lengths() {
    check(
        r#"<template v-match="subject"><p v-when="[]">empty</p><p v-when="[1,]">one</p><p v-when="[1, _, ...]">rest</p><p v-when="_">other</p></template>"#,
        json!([
            {"context": {"subject": []}, "trees": [paragraph("empty")]},
            {"context": {"subject": [1]}, "trees": [paragraph("one")]},
            {"context": {"subject": [1, 2]}, "trees": [paragraph("rest")]},
            {"context": {"subject": [1, 2, 3]}, "trees": [paragraph("rest")]},
            {"context": {"subject": {}}, "trees": [paragraph("other")]}
        ]),
    );
}

#[test]
fn array_patterns_check_length_before_reading_elements() {
    for pattern in ["[1, 2]", "[1, 2, ...]"] {
        check(
            &format!(
                r#"<template v-match="subject"><p v-when="{pattern}">matched</p><p v-when="_">other</p></template>"#
            ),
            json!([{"scenario": "short-array-getter", "trees": [paragraph("other")], "reads": 0}]),
        );
    }
}

#[test]
fn match_preserves_authored_container_and_updates_its_branch() {
    let tree = |state: &str, text: &str| {
        json!([{
            "tag": "section", "attributes": {"class": "shell", "data-state": state},
            "children": paragraph(text)
        }])
    };
    check(
        r#"<section class="shell" :data-state="subject" v-match="subject"><p v-when="'ready'">Ready</p><p v-when="_">Other</p></section>"#,
        json!([{"context": {"subject": "ready"}, "steps": [{"patch": {"subject": "waiting"}}, {"patch": {"subject": "ready"}}], "trees": [tree("ready", "Ready"), tree("waiting", "Other"), tree("ready", "Ready")]}]),
    );
}

#[test]
fn match_preserves_hosts_inside_authored_loop_scopes() {
    let section = |id: i32, text: &str| {
        json!({
            "tag": "section", "attributes": {"data-id": id.to_string()}, "children": paragraph(text)
        })
    };
    check(
        r#"<section v-for="item in items" :key="item.id" :data-id="item.id" v-match="item.state"><p v-when="'ready'">Ready</p><p v-when="_">Other</p></section>"#,
        json!([{"context": {"items": [{"id": 1, "state": "ready"}, {"id": 2, "state": "waiting"}]},
            "steps": [{"patch": {"items": [{"id": 2, "state": "ready"}, {"id": 1, "state": "waiting"}]}}],
            "trees": [[section(1, "Ready"), section(2, "Other")], [section(2, "Ready"), section(1, "Other")]]}]),
    );
}

#[test]
fn object_patterns_check_numeric_and_quoted_property_keys() {
    check(
        r#"<template v-match="subject"><p v-when="{ 0: _, 'a-b': _ }">present</p><p v-when="_">absent</p></template>"#,
        json!([
            {"context": {"subject": {"0": null, "a-b": null}}, "trees": [paragraph("present")]},
            {"context": {"subject": {"0": null}}, "trees": [paragraph("absent")]},
            {"context": {"subject": {"a-b": null}}, "trees": [paragraph("absent")]}
        ]),
    );
}

#[test]
fn object_patterns_preserve_unicode_static_property_keys() {
    for (pattern, content) in [
        ("{ \u{e9}: _ }", "present"),
        ("{ \u{e9}: 1 }", "present"),
        (
            "{ \u{e9}: const value }",
            "{{ value === 1 ? 'present' : 'wrong' }}",
        ),
    ] {
        check(
            &format!(
                r#"<template v-match="subject"><p v-when="{pattern}">{content}</p><p v-when="_">absent</p></template>"#
            ),
            json!([
                {"context": {"subject": {"\u{e9}": 1}, "\u{e9}": "wrong"}, "trees": [paragraph("present")]},
                {"context": {"subject": {"wrong": 1}, "\u{e9}": "wrong"}, "trees": [paragraph("absent")]},
                {"context": {"subject": {}}, "trees": [paragraph("absent")]}
            ]),
        );
    }
}
