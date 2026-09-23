//! P3-4 native `v-model` reference contract through both mounted runtimes.
//!
//! Every case in `model-reference.cases.jsonl` drives user events and reactive
//! patches through the compiled DOM and Vapor output. Each full observation,
//! including live `value`, `checked` and `selected` state, must equal the
//! independent Lean model reference in `model-reference.behavior.jsonl`.

use crate::{Scenario, mounted_trace_with_identity};
use serde_json::Value;
use std::path::Path;

fn lines(name: &str) -> Vec<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/formal/impeto/fixtures")
        .join(name);
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn run_family(prefixes: &[&str]) {
    run_stem("model-reference", prefixes, false, 10);
}

fn run_stem(stem: &str, prefixes: &[&str], identities: bool, minimum: usize) {
    let cases = lines(&format!("{stem}.cases.jsonl"));
    let behaviors = lines(&format!("{stem}.behavior.jsonl"));
    assert_eq!(cases.len(), behaviors.len());
    assert!(cases.len() >= minimum, "{stem} is too small");
    let mut checked = 0;
    for (case, behavior) in cases.iter().zip(&behaviors) {
        let name = case["name"].as_str().unwrap();
        assert_eq!(
            behavior["name"], case["name"],
            "model files are out of step"
        );
        if !prefixes.iter().any(|prefix| name.starts_with(prefix)) {
            continue;
        }
        checked += 1;
        let scenario: Scenario = serde_json::from_value(case["scenario"].clone()).unwrap();
        let expected = behavior["trace"].as_array().unwrap();
        for backend in ["vdom", "vapor"] {
            let actual = mounted_trace_with_identity(
                backend,
                case["template"].as_str().unwrap(),
                Value::Object(scenario.context.clone()),
                Value::Array(scenario.steps.clone()),
                false,
                identities,
            );
            let actual = actual.as_array().unwrap();
            assert_eq!(actual.len(), expected.len(), "{name}: {backend} length");
            for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
                assert_eq!(actual, expected, "{name}: {backend} observation {index}");
            }
        }
    }
    assert!(checked >= 3, "{prefixes:?}: model family is too small");
}

#[test]
fn mounted_text_models_match_the_lean_model_reference() {
    run_family(&["text-", "member-", "recreated-"]);
}

#[test]
fn mounted_toggle_models_match_the_lean_model_reference() {
    run_family(&["checkbox-", "radio-"]);
}

#[test]
fn mounted_select_models_match_the_lean_model_reference() {
    run_family(&["select-"]);
}

/// Keyed interactions: element identity decides which item an in-flight IME
/// commit or a checkbox toggle reaches when rows reorder around it.
#[test]
fn mounted_looped_models_match_the_lean_model_reference() {
    run_stem("model-loop-reference", &["keyed-", "positional-"], true, 4);
}
