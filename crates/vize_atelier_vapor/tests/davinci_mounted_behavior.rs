//! TS-30 mounted behavior through the published Vue DOM and Vapor runtimes.

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
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn mounted_trace(backend: &str, source: &str, context: Value, steps: Value) -> Value {
    let allocator = Allocator::new();
    let code = if backend == "vdom" {
        let (_, errors, result) = compile_template_with_options(
            &allocator,
            source,
            DomCompilerOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert!(errors.is_empty(), "DOM errors: {errors:?}");
        format!("{}\n{}", result.preamble, result.code)
    } else {
        let result = compile_vapor(
            &allocator,
            source,
            VaporCompilerOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert!(
            result.error_messages.is_empty(),
            "Vapor errors: {:?}",
            result.error_messages
        );
        result.code.to_string()
    };
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/davinci-mounted-trace.mjs");
    let mut child = Command::new("node")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start mounted runtime runner (install workspace JS dependencies first)");
    let input = json!({ "backend": backend, "code": code, "context": context, "steps": steps });
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{backend} mounted runner failed:\n{}\ncode:\n{code}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
}

fn assert_backends(source: &str, context: Value, steps: Value) -> Value {
    let dom = mounted_trace("vdom", source, context.clone(), steps.clone());
    let vapor = mounted_trace("vapor", source, context, steps);
    assert_eq!(dom, vapor, "mounted backend behavior diverged for {source}");
    dom
}

#[test]
fn mounted_dynamic_button_updates_and_dispatches_events() {
    let trace = assert_backends(
        r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#,
        json!({"locked": true, "label": "Save"}),
        json!([{"patch": {"locked": false, "label": "Publish"}}, {"event": "click", "selector": "button"}]),
    );
    assert_eq!(trace[0]["tree"][0]["children"][0]["disabled"], true);
    assert_eq!(trace[1]["tree"][0]["children"][0]["disabled"], false);
    assert_eq!(
        trace[1]["tree"][0]["children"][0]["children"],
        json!(["Publish"])
    );
    assert_eq!(trace[2]["events"], json!(["save"]));
    assert_eq!(trace[3]["tree"], json!([]));
}

#[test]
fn mounted_branch_and_slot_fallback_update_together() {
    let trace = assert_backends(
        r#"<section><p v-if="ready">ready</p><slot name="body"><span v-text="fallback"></span></slot></section>"#,
        json!({"ready": true, "fallback": "fallback"}),
        json!([{"patch": {"ready": false, "fallback": "waiting"}}, {"patch": {"ready": true, "fallback": "done"}}]),
    );
    assert_eq!(trace[0]["tree"][0]["children"].as_array().unwrap().len(), 2);
    assert_eq!(trace[1]["tree"][0]["children"].as_array().unwrap().len(), 1);
    assert_eq!(
        trace[1]["tree"][0]["children"][0]["children"],
        json!(["waiting"])
    );
    assert_eq!(
        trace[2]["tree"][0]["children"][1]["children"],
        json!(["done"])
    );
}

#[test]
fn mounted_v_text_handles_empty_elements_and_multiple_effects() {
    for source in [
        r#"<span v-text="label"></span>"#,
        r#"<span :class="label" v-text="label"></span>"#,
        r#"<section><span v-text="label"></span></section>"#,
    ] {
        let trace = assert_backends(
            source,
            json!({"label": "start"}),
            json!([{"patch": {"label": "updated"}}, {"patch": {"label": null}}, {"patch": {"label": 42}}]),
        );
        let rendered = |index: usize| {
            let root = &trace[index]["tree"][0];
            if root["tag"] == "section" {
                root["children"][0]["children"].clone()
            } else {
                root["children"].clone()
            }
        };
        assert_eq!(rendered(0), json!(["start"]));
        assert_eq!(rendered(1), json!(["updated"]));
        assert_eq!(rendered(2), json!([]));
        assert_eq!(rendered(3), json!(["42"]));
    }
}

#[test]
fn mounted_slot_before_branch_preserves_authored_order() {
    let trace = assert_backends(
        r#"<section><slot name="body"><span v-text="fallback"></span></slot><p v-if="ready">ready</p></section>"#,
        json!({"ready": true, "fallback": "first"}),
        json!([{"patch": {"ready": false}}, {"patch": {"ready": true}}]),
    );
    for index in [0, 2] {
        assert_eq!(trace[index]["tree"][0]["children"][0]["tag"], "span");
        assert_eq!(trace[index]["tree"][0]["children"][1]["tag"], "p");
    }
}
