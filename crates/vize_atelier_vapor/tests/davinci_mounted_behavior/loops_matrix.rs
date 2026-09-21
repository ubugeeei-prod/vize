//! TS-29 IVM matrix through the mounted Vue DOM and Vapor runtimes.
//!
//! Each case in `ivm-matrix.cases.jsonl` is compiled by both backends and
//! driven through its scenario. Every observation (tree, delivered events and
//! actual element lifetimes) must equal the Lean reference's from-scratch
//! render in `ivm-matrix.behavior.jsonl`.
//!
//! One upstream defect is pinned exactly instead of hidden:
//! `@vue/runtime-vapor` 3.6.0-beta.10 reuses positional `createFor` blocks
//! with `update(block, getItem(source, i)[0])`, so an unkeyed object loop keeps
//! each block's stale key alias after keys shift. The compiled Vapor reads
//! `_for_key0.value` reactively, and VDOM matches the reference. Only
//! `object-positional-*` Vapor traces may appear in `ivm-matrix.vapor-gaps.jsonl`.
//! Each entry must still differ from the reference and must be reproduced
//! exactly. Once upstream fixes the defect, this test fails until the entry is
//! deleted. `VIZE_UPDATE_IVM_VAPOR_GAPS=1` re-records only that family.

use crate::{Scenario, mounted_trace_with_identity};
use serde_json::Value;
use std::path::{Path, PathBuf};

const GAP_FAMILY: &str = "object-positional-";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../formal/impeto/fixtures")
        .join(name)
}

fn lines(name: &str) -> Vec<Value> {
    std::fs::read_to_string(fixture(name))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

/// The pinned gap class: traces may differ only in `data-id` spellings and the
/// matching identity labels, never in structure, text, events or lifetimes.
fn differs_only_in_key_labels(gap: &Value, reference: &Value, field: &str) -> bool {
    match (gap, reference) {
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter().all(|(key, value)| {
                    b.get(key)
                        .is_some_and(|other| differs_only_in_key_labels(value, other, key))
                })
        }
        (Value::Array(a), Value::Array(b)) if field == "identities" => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(x, y)| x[1] == y[1] && x[0].is_string())
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(x, y)| differs_only_in_key_labels(x, y, field))
        }
        (Value::String(_), Value::String(_)) if field == "data-id" => true,
        _ => gap == reference,
    }
}

fn run_family(prefix: &str) {
    let cases = lines("ivm-matrix.cases.jsonl");
    let behaviors = lines("ivm-matrix.behavior.jsonl");
    let record =
        prefix == GAP_FAMILY && std::env::var("VIZE_UPDATE_IVM_VAPOR_GAPS").as_deref() == Ok("1");
    let gaps = if record {
        Vec::new()
    } else {
        lines("ivm-matrix.vapor-gaps.jsonl")
    };
    let mut recorded = String::new();
    assert_eq!(cases.len(), behaviors.len());
    if !record {
        // Exactly the pinned class, in matrix order: every unkeyed object case.
        let pinned: Vec<&Value> = cases
            .iter()
            .map(|case| &case["name"])
            .filter(|name| name.as_str().unwrap().starts_with(GAP_FAMILY))
            .collect();
        let listed: Vec<&Value> = gaps.iter().map(|gap| &gap["name"]).collect();
        assert_eq!(
            listed, pinned,
            "gap entries must be exactly the pinned class"
        );
    }
    let mut checked = 0;
    for (case, behavior) in cases.iter().zip(&behaviors) {
        let name = case["name"].as_str().unwrap();
        assert_eq!(
            behavior["name"], case["name"],
            "matrix files are out of step"
        );
        if !name.starts_with(prefix) {
            continue;
        }
        let scenario: Scenario = serde_json::from_value(case["scenario"].clone()).unwrap();
        for backend in ["vdom", "vapor"] {
            let gap = gaps
                .iter()
                .find(|gap| backend == "vapor" && gap["name"] == case["name"]);
            if let Some(gap) = gap {
                assert_ne!(gap["trace"], behavior["trace"], "{name}: stale gap entry");
                assert!(
                    differs_only_in_key_labels(&gap["trace"], &behavior["trace"], ""),
                    "{name}: gap entry exceeds the stale key-alias class"
                );
            }
            let expected = gap.unwrap_or(behavior)["trace"].as_array().unwrap();
            let actual = mounted_trace_with_identity(
                backend,
                case["template"].as_str().unwrap(),
                Value::Object(scenario.context.clone()),
                Value::Array(scenario.steps.clone()),
                false,
                true,
            );
            if record && backend == "vapor" && actual != behavior["trace"] {
                let entry = serde_json::json!({ "name": name, "trace": actual });
                recorded.push_str(&format!("{entry}\n"));
                continue;
            }
            let actual = actual.as_array().unwrap();
            assert_eq!(actual.len(), expected.len(), "{name}: {backend} length");
            for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
                assert_eq!(actual, expected, "{name}: {backend} observation {index}");
            }
        }
        checked += 1;
    }
    assert!(checked >= 4, "{prefix}: matrix family is too small");
    if record {
        std::fs::write(fixture("ivm-matrix.vapor-gaps.jsonl"), recorded).unwrap();
    }
}

#[test]
fn array_keyed_matrix_matches_reference() {
    run_family("array-keyed-");
}

#[test]
fn array_positional_matrix_matches_reference() {
    run_family("array-positional-");
}

#[test]
fn object_keyed_matrix_matches_reference() {
    run_family("object-keyed-");
}

#[test]
fn object_positional_matrix_matches_reference_or_pinned_upstream_gap() {
    run_family(GAP_FAMILY);
}

#[test]
fn range_keyed_matrix_matches_reference() {
    run_family("range-keyed-");
}

#[test]
fn range_positional_matrix_matches_reference() {
    run_family("range-positional-");
}
