use crate::{Scenario, mounted_trace_with_identity};
use serde_json::Value;
use std::path::Path;

#[test]
fn mounted_loops_match_reference_trees_identities_and_fresh_events() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../formal/impeto/fixtures");
    for name in ["keyed", "unkeyed", "nested"] {
        let stem = fixtures.join(format!("rust-lowered-loop-{name}"));
        let source = std::fs::read_to_string(stem.with_extension("template.txt")).unwrap();
        let scenario: Scenario = serde_json::from_str(
            &std::fs::read_to_string(stem.with_extension("scenario.json")).unwrap(),
        )
        .unwrap();
        let expected: Value = serde_json::from_str(
            &std::fs::read_to_string(stem.with_extension("behavior.json")).unwrap(),
        )
        .unwrap();
        for backend in ["vdom", "vapor"] {
            let actual = mounted_trace_with_identity(
                backend,
                source.trim_end(),
                Value::Object(scenario.context.clone()),
                Value::Array(scenario.steps.clone()),
                false,
                true,
            );
            assert_eq!(
                actual.as_array().unwrap().len(),
                expected.as_array().unwrap().len()
            );
            for (index, (actual, expected)) in actual
                .as_array()
                .unwrap()
                .iter()
                .zip(expected.as_array().unwrap())
                .enumerate()
            {
                assert_eq!(actual, expected, "{name}: {backend} observation {index}");
            }
        }
    }
}

#[test]
fn native_loop_function_handlers_execute_once_and_keep_current_alias_values() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../formal/impeto/fixtures/rust-lowered-loop-keyed");
    let source = std::fs::read_to_string(fixture.with_extension("template.txt")).unwrap();
    let scenario: Scenario = serde_json::from_str(
        &std::fs::read_to_string(fixture.with_extension("scenario.json")).unwrap(),
    )
    .unwrap();
    let expected: Value = serde_json::from_str(
        &std::fs::read_to_string(fixture.with_extension("behavior.json")).unwrap(),
    )
    .unwrap();
    for handler in [
        "$event => record(item.label)",
        "() => record(item.label)",
        "function () { record(item.label) }",
        "record?.(item.label)",
    ] {
        let source = source.replace("record(item.label)", handler);
        for backend in ["vdom", "vapor"] {
            let actual = mounted_trace_with_identity(
                backend,
                source.trim_end(),
                Value::Object(scenario.context.clone()),
                Value::Array(scenario.steps.clone()),
                false,
                true,
            );
            for (index, (actual, expected)) in actual
                .as_array()
                .unwrap()
                .iter()
                .zip(expected.as_array().unwrap())
                .enumerate()
            {
                assert_eq!(
                    actual, expected,
                    "{backend}: {handler}, observation {index}"
                );
            }
            assert_eq!(
                actual.as_array().unwrap().len(),
                expected.as_array().unwrap().len()
            );
        }
    }
}
