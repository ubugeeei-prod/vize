//! Whole original CSS engine defects and independently reviewed public outputs.
#![cfg(test)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use vize_glyph::{FormatError, FormatOptions, format_sfc, format_style};
use vize_l0::{String, cstr};

const CORPUS: &str =
    include_str!("../../../tests/_fixtures/differential/css-engine-boundary-3295/cases.json");
const ORIGINAL: &str =
    include_str!("../../../tests/_fixtures/differential/css-engine-boundary-3295/App.vue.txt");
const EXPECTED: &str =
    include_str!("../../../tests/_fixtures/differential/css-engine-boundary-3295/App.expected.txt");

fn corpus() -> Value {
    let mut hash = String::with_capacity(64);
    for byte in Sha256::digest(CORPUS.as_bytes()) {
        hash.push_str(cstr!("{byte:02x}").as_str());
    }
    assert_eq!(
        hash,
        "32b8bb5632bc0755d1540ff828f3e70eb5e41ec45cb0c81de4df67a787f572c1"
    );
    let corpus: Value = serde_json::from_str(CORPUS).expect("finite complete CSS boundary corpus");
    assert_eq!(corpus["issue"], 3295);
    assert_eq!(corpus["relatedIssue"], 4961);
    assert_eq!(corpus["formatterCases"].as_array().unwrap().len(), 12);
    assert_eq!(corpus["styleCases"].as_array().unwrap().len(), 6);
    let originals = corpus["originals"].as_array().unwrap();
    assert_eq!(originals.len(), 3);
    for (case, css) in originals.iter().zip([
        "a{opacity:abs(-50%)}",
        "a{color:hsl(0 abs(-50%) 0%)}",
        "a{text-size-adjust:calc(5)}",
    ]) {
        assert_eq!(case["source"], css, "complete unmodified original CSS");
    }
    corpus
}

fn successor(case: &Value, surface: &str) -> Option<Value> {
    let authority: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/css-engine-vendor-exit/finite-successors.json"
    ))
    .unwrap();
    assert_eq!(authority["rustCases"].as_array().unwrap().len(), 11);
    let mut rows = authority["rustCases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["surface"] == surface && row["case"] == case["id"]);
    let row = rows.next()?.clone();
    assert!(rows.next().is_none(), "one named successor");
    assert_eq!(row["source"], case["source"], "whole frozen original input");
    Some(row)
}

fn retain(name: &str, observations: &[Value]) {
    let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".into());
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile)
        .join("css-engine-boundary-3295-glyph");
    fs::create_dir_all(&directory).expect("existing hosted API capture directory");
    fs::write(
        directory.join(name),
        serde_json::to_vec_pretty(observations).unwrap(),
    )
    .expect("retain complete actual public API observations");
}

fn sfc_passes(case: &Value, observations: &mut Vec<Value>, capture_name: &str) {
    let options: FormatOptions = serde_json::from_value(case["options"].clone()).unwrap();
    let repaired = successor(case, "formatter");
    let expected = repaired
        .as_ref()
        .map_or(&case["expected"], |row| &row["wholeCode"])
        .as_str()
        .unwrap();
    let mut current = String::from(case["source"].as_str().unwrap());
    for pass in 1..=3 {
        let result = format_sfc(&current, &options)
            .expect("complete SFC result including the finite repaired cases");
        observations.push(json!({"case": case["id"], "pass": pass,
            "source": current, "options": case["options"], "beforeExpected": case["expected"], "expected": expected,
            "actual": {"code": result.code, "changed": result.changed}}));
        retain(capture_name, observations);
        assert_eq!(result.code.as_str(), expected, "{} pass {pass}", case["id"]);
        assert_eq!(result.changed, current != expected, "whole changed flag");
        current = result.code;
    }
}

#[test]
fn original_css_engine_defects_keep_complete_sfc_outputs_and_fixpoints() {
    let corpus = corpus();
    let case = &corpus["formatterCases"][3];
    assert_eq!(case["id"], "original-combined");
    assert_eq!(case["source"], ORIGINAL);
    assert_eq!(case["expected"], EXPECTED);
    let mut observations = Vec::new();
    sfc_passes(case, &mut observations, "whole-original-sfc.json");
    assert_eq!(observations.len(), 3);
    retain("whole-original-sfc.json", &observations);
}

#[test]
fn separate_originals_healthy_neighbors_and_invalid_css_keep_whole_sfc_references() {
    let corpus = corpus();
    let mut observations = Vec::new();
    for case in corpus["formatterCases"].as_array().unwrap() {
        if case["id"] != "original-combined" {
            sfc_passes(case, &mut observations, "whole-sfc-controls.json");
        }
    }
    assert_eq!(observations.len(), 33);
    retain("whole-sfc-controls.json", &observations);
}

#[test]
fn public_style_errors_are_complete_and_do_not_prevent_later_healthy_fixedpoints() {
    let corpus = corpus();
    let options = FormatOptions::default();
    let mut observations = Vec::new();
    for case in corpus["styleCases"].as_array().unwrap() {
        let repaired = successor(case, "style");
        let expected = repaired
            .as_ref()
            .map_or(&case["expected"], |row| &row["wholeExpected"]["code"]);
        let mut current = String::from(case["source"].as_str().unwrap());
        for pass in 1..=3 {
            match format_style(&current, &options) {
                Ok(actual) => {
                    observations.push(json!({"case": case["id"], "pass": pass,
                        "source": current, "actual": {"code": actual}}));
                    retain("whole-style-results.json", &observations);
                    assert!(
                        repaired.is_some() || case.get("error").is_none(),
                        "unchanged invalid CSS must return its whole error"
                    );
                    assert_eq!(Some(actual.as_str()), expected.as_str());
                    current = actual;
                }
                Err(error) => {
                    assert!(
                        repaired.is_none(),
                        "named repaired original must return complete CSS"
                    );
                    let display = cstr!("{error}");
                    let (variant, message) = match &error {
                        FormatError::StyleFormatError(message) => {
                            ("StyleFormatError", Some(message.as_str()))
                        }
                        _ => ("UnexpectedFormatError", None),
                    };
                    observations.push(json!({"case": case["id"], "pass": pass,
                        "source": current, "actual": {"variant": variant,
                            "message": message, "display": display}}));
                    retain("whole-style-results.json", &observations);
                    assert_eq!(Some(display.as_str()), case["displayError"].as_str());
                    let FormatError::StyleFormatError(message) = error else {
                        panic!("public CSS defect must retain StyleFormatError");
                    };
                    assert_eq!(Some(message.as_str()), case["error"].as_str());
                }
            }
        }
    }
    assert_eq!(observations.len(), 18);
    retain("whole-style-results.json", &observations);
}
