//! Custom directive payloads and lifecycle match the published Vapor runtime.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    reason = "runtime comparison fixtures use std strings and assert by panicking"
)]

use serde_json::json;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn trace(runner: &str, input: serde_json::Value) -> Vec<serde_json::Value> {
    use std::{
        io::Write,
        path::Path,
        process::{Command, Stdio},
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support")
        .join(runner);
    let mut child = Command::new("node")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{runner}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn custom_directive_payloads_remain_reactive_on_elements_and_components() {
    let child = r#"<p>{{ label }}</p>"#;
    let second = r#"<section>{{ label }}</section>"#;
    for (case, source, directives) in [
        (
            "element",
            r#"<div v-focus:top.lazy="value">{{ value }}</div>"#,
            vec!["focus"],
        ),
        (
            "component",
            r#"<Child v-focus:top.lazy="value" :label="value" />"#,
            vec!["focus"],
        ),
        (
            "dynamic",
            r#"<component :is="view" v-focus:[placement].lazy="value" :label="value" />"#,
            vec!["focus"],
        ),
        (
            "multiple",
            r#"<Child v-focus:[placement]="value" v-track:[placement].deep="other" :label="value" />"#,
            vec!["focus", "track"],
        ),
        ("absent", r#"<Child v-focus />"#, vec!["focus"]),
        ("modifiers", r#"<Child v-focus.lazy />"#, vec!["focus"]),
        (
            "hyphen",
            r#"<Child v-focus.foo-bar="value" :label="value" />"#,
            vec!["focus"],
        ),
    ] {
        let allocator = Allocator::new();
        let options = VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        };
        let parent = compile_vapor(&allocator, source, options.clone());
        let child_code = compile_vapor(&allocator, child, options);
        let second_code = compile_vapor(
            &allocator,
            second,
            VaporCompilerOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        for compiled in [&parent, &child_code, &second_code] {
            assert!(
                compiled.error_messages.is_empty(),
                "{case}: {:?}",
                compiled.error_messages
            );
        }
        let context = json!({"view": "Child", "value": "A", "other": 1, "placement": "top"});
        let steps = json!([
            {"patch": {"value": "B", "other": 2, "placement": "bottom"}},
            {"patch": {"value": "C", "other": 3}},
            {"patch": {"view": "Second", "value": "D"}},
            {"patch": {"view": "Child", "value": "E"}},
        ]);
        let actual = trace(
            "davinci-directive-trace.mjs",
            json!({
                "code": parent.code, "context": context, "steps": steps, "directives": directives,
            "components": {"Child": {"code": child_code.code, "props": ["label"]}, "Second": {"code": second_code.code, "props": ["label"]}},
            }),
        );
        let expected = trace(
            "davinci-directive-trace.mjs",
            json!({
                "source": source, "context": context, "steps": steps, "directives": directives,
            "components": {"Child": {"source": child, "props": ["label"]}, "Second": {"source": second, "props": ["label"]}},
            }),
        );
        assert_eq!(
            actual, expected,
            "{case}: complete DOM and lifecycle traces"
        );
        insta::assert_debug_snapshot!(format!("directive_{case}"), actual);
        insta::assert_snapshot!(format!("directive_{case}_code"), parent.code);
    }
}
