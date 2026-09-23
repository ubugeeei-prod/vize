//! P3-4 supplied-slot reference contract through both mounted runtimes.
//!
//! Every case in `slot-reference.cases.jsonl` mounts the compiled template
//! under a caller that supplies its slots (static text or one displayed slot
//! prop). Each full observation, including element lifetimes, must equal the
//! independent Lean reference in `slot-reference.behavior.jsonl`.

use crate::davinci_mounted_behavior::runtime::mounted_trace_with_slots;
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

#[test]
fn mounted_supplied_slots_match_the_lean_reference() {
    let cases = lines("slot-reference.cases.jsonl");
    let behaviors = lines("slot-reference.behavior.jsonl");
    assert_eq!(cases.len(), behaviors.len());
    assert!(cases.len() >= 8, "slot reference is too small");
    for (case, behavior) in cases.iter().zip(&behaviors) {
        let name = case["name"].as_str().unwrap();
        assert_eq!(behavior["name"], case["name"], "slot files are out of step");
        let scenario = &case["scenario"];
        let expected = behavior["trace"].as_array().unwrap();
        for backend in ["vdom", "vapor"] {
            let actual = mounted_trace_with_slots(
                backend,
                case["template"].as_str().unwrap(),
                scenario["context"].clone(),
                scenario["steps"].clone(),
                scenario["slots"].clone(),
            );
            let actual = actual.as_array().unwrap();
            assert_eq!(actual.len(), expected.len(), "{name}: {backend} length");
            for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
                assert_eq!(actual, expected, "{name}: {backend} observation {index}");
            }
        }
    }
}
