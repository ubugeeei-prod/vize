use std::{fs, path::PathBuf};

use serde_json::Value;

fn repository_file(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn count_open_object_schemas(value: &Value) -> usize {
    let current = usize::from(
        value.get("type") == Some(&Value::String("object".into()))
            && value.get("additionalProperties") != Some(&Value::Bool(false)),
    );

    current
        + match value {
            Value::Array(values) => values.iter().map(count_open_object_schemas).sum(),
            Value::Object(values) => values.values().map(count_open_object_schemas).sum(),
            _ => 0,
        }
}

#[test]
fn test_run_schema_is_strict_and_versioned() {
    let schema: Value = serde_json::from_slice(
        &fs::read(repository_file(
            "crates/vize_marquette/schema/test-run-evidence.schema.json",
        ))
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(
        schema["$defs"]["evidence"]["properties"]["format"]["const"],
        "vize.test-run.evidence"
    );
    assert_eq!(
        schema["$defs"]["evidence"]["properties"]["formatVersion"]["const"],
        1
    );
    assert!(
        schema["$defs"]["timestamp"]["pattern"]
            .as_str()
            .unwrap()
            .ends_with("Z$")
    );

    assert_eq!(
        count_open_object_schemas(&schema),
        0,
        "every object schema must reject unknown properties"
    );
}

#[test]
fn admission_schema_is_strict_and_documents_the_append_only_policy() {
    let schema: Value = serde_json::from_slice(
        &fs::read(repository_file(
            "crates/vize_marquette/schema/test-run-admission.schema.json",
        ))
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    let description = schema["$defs"]["denialCode"]["description"]
        .as_str()
        .unwrap();
    assert!(description.contains("append-only"));
    assert!(description.contains("never renamed, renumbered, reused, or removed"));

    let codes = schema["$defs"]["denialCode"]["enum"].as_array().unwrap();
    let mut sorted = codes.clone();
    sorted.sort_by_key(|value| value.as_str().unwrap().to_owned());
    assert_eq!(
        codes, &sorted,
        "denial codes must stay in lexicographic order"
    );

    assert_eq!(
        count_open_object_schemas(&schema),
        0,
        "every object schema must reject unknown properties"
    );
}

#[test]
fn check_schema_is_strict_and_rejects_generic_references() {
    let schema: Value = serde_json::from_slice(
        &fs::read(repository_file(
            "crates/vize_marquette/schema/test-run-check.schema.json",
        ))
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(
        schema["$defs"]["check"]["properties"]["format"]["const"],
        "vize.test-run.check"
    );
    assert_eq!(
        schema["$defs"]["check"]["properties"]["formatVersion"]["const"],
        1
    );
    assert_eq!(
        schema["$defs"]["admissionId"]["pattern"], "^test-run:[a-f0-9]{64}$",
        "tests evidence must be an exact admission id, never a generic reference"
    );

    assert_eq!(
        count_open_object_schemas(&schema),
        0,
        "every object schema must reject unknown properties"
    );
}
