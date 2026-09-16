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

#[test]
fn mounted_text_model_defers_composition_until_commit() {
    let trace = assert_backends(
        r#"<section><input v-model="value"><output>{{ value }}</output></section>"#,
        json!({"value": "initial"}),
        json!([
            {"event": "compositionstart", "selector": "input"},
            {"event": "input", "selector": "input", "value": "\u{306b}\u{307b}\u{3093}"},
            {"event": "compositionend", "selector": "input", "value": "\u{65e5}\u{672c}"},
            {"patch": {"value": "external"}}
        ]),
    );
    assert_eq!(
        trace[2]["tree"][0]["children"][1]["children"],
        json!(["initial"])
    );
    assert_eq!(
        trace[3]["tree"][0]["children"][1]["children"],
        json!(["\u{65e5}\u{672c}"])
    );
    assert_eq!(trace[4]["tree"][0]["children"][0]["value"], "external");
}

#[test]
fn mounted_lazy_text_model_commits_on_change() {
    let trace = assert_backends(
        r#"<section><input v-model.lazy="value"><output>{{ value }}</output></section>"#,
        json!({"value": "initial"}),
        json!([
            {"event": "input", "selector": "input", "value": "pending"},
            {"event": "change", "selector": "input", "value": "committed"}
        ]),
    );
    assert_eq!(
        trace[1]["tree"][0]["children"][1]["children"],
        json!(["initial"])
    );
    assert_eq!(
        trace[2]["tree"][0]["children"][1]["children"],
        json!(["committed"])
    );
}

#[test]
fn mounted_trim_number_model_preserves_numeric_values() {
    let trace = assert_backends(
        r#"<section><input v-model.trim.number="value"><output>{{ typeof value }}:{{ value }}</output></section>"#,
        json!({"value": 0}),
        json!([{"event": "input", "selector": "input", "value": " 42 "}, {"event": "change", "selector": "input"}]),
    );
    assert_eq!(
        trace[1]["tree"][0]["children"][1]["children"],
        json!(["number:42"])
    );
    assert_eq!(trace[2]["tree"][0]["children"][0]["value"], "42");
}

#[test]
fn mounted_checkbox_model_updates_array_membership() {
    let trace = assert_backends(
        r#"<section><input type="checkbox" value="b" v-model="values"><output>{{ values.join(',') }}</output></section>"#,
        json!({"values": ["a"]}),
        json!([{"event": "change", "selector": "input", "checked": true}, {"event": "change", "selector": "input", "checked": false}, {"patch": {"values": ["b"]}}]),
    );
    assert_eq!(
        trace[1]["tree"][0]["children"][1]["children"],
        json!(["a,b"])
    );
    assert_eq!(trace[2]["tree"][0]["children"][1]["children"], json!(["a"]));
    assert_eq!(trace[3]["tree"][0]["children"][0]["checked"], true);
}

#[test]
fn mounted_select_multiple_model_reconciles_selection() {
    let trace = assert_backends(
        r#"<section><select multiple v-model="values"><option value="a">A</option><option value="b">B</option></select><output>{{ values.join(',') }}</output></section>"#,
        json!({"values": ["a"]}),
        json!([{"event": "change", "selector": "select", "selectedValues": ["a", "b"]}, {"patch": {"values": ["b"]}}]),
    );
    assert_eq!(
        trace[1]["tree"][0]["children"][1]["children"],
        json!(["a,b"])
    );
    assert_eq!(
        trace[2]["tree"][0]["children"][0]["children"][0]["selected"],
        false
    );
    assert_eq!(
        trace[2]["tree"][0]["children"][0]["children"][1]["selected"],
        true
    );
}

#[test]
fn mounted_model_resolves_member_and_computed_bindings() {
    for binding in ["form.value", "form[key]"] {
        let source = format!(
            r#"<section><input v-model="{binding}"><output>{{{{ form.value }}}}</output></section>"#
        );
        let trace = assert_backends(
            &source,
            json!({"form": {"value": "initial"}, "key": "value"}),
            json!([{"event": "input", "selector": "input", "value": "typed"}, {"patch": {"form": {"value": "external"}}}]),
        );
        assert_eq!(trace[0]["tree"][0]["children"][0]["value"], "initial");
        assert_eq!(
            trace[1]["tree"][0]["children"][1]["children"],
            json!(["typed"])
        );
        assert_eq!(trace[2]["tree"][0]["children"][0]["value"], "external");
    }
}

#[test]
fn model_codegen_resolves_identifiers_with_and_without_prefixing() {
    for prefix_identifiers in [false, true] {
        let allocator = Allocator::new();
        let result = compile_vapor(
            &allocator,
            r#"<input v-model="value">"#,
            VaporCompilerOptions {
                prefix_identifiers,
                ..Default::default()
            },
        );
        assert!(result.error_messages.is_empty());
        insta::assert_snapshot!(
            format!("native_model_prefix_{prefix_identifiers}"),
            result.code
        );
    }
}
