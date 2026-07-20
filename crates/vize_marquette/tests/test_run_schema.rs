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
