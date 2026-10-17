use serde_json::{Value, json};

use super::test_support::{
    close_project_request, exchange, frames, initialize_request, open_project_request,
    transform_request,
};

#[test]
fn inferred_project_returns_dynamic_config_identity_without_file_discovery() {
    let input = frames(&[
        initialize_request(),
        open_project_request(2, json!({ "optionsApi": true }), json!({})),
    ]);
    let responses = exchange(&input);

    assert!(
        responses[1]["result"]["configIdentity"]
            .as_str()
            .unwrap()
            .starts_with("vize-content-mapper-v1:")
    );
    assert!(responses[1]["result"].get("watchedFiles").is_none());
    assert!(responses[1]["result"].get("optionDiagnostics").is_none());
}

#[test]
fn rejects_open_project_before_initialize() {
    let input = frames(&[open_project_request(1, Value::Null, json!({}))]);
    let responses = exchange(&input);

    assert_eq!(responses[0]["error"]["code"], -32002);
}

#[test]
fn unknown_options_are_reported_as_option_diagnostics() {
    let source = r#"<script lang="ts">
export default { data() { return { count: 1 } } }
</script>
<template>{{ count }}</template>
"#;
    let input = frames(&[
        initialize_request(),
        open_project_request(2, json!({ "optionApi": true }), json!({})),
        transform_request(3, source),
    ]);
    let responses = exchange(&input);

    assert_eq!(
        responses[1]["result"]["optionDiagnostics"],
        json!([{
            "path": ["optionApi"],
            "messageText": "Unknown option 'optionApi'",
            "code": 2
        }])
    );
    // The unknown option degrades to defaults instead of failing the project.
    assert!(
        responses[2]["result"]["text"]
            .as_str()
            .unwrap()
            .contains("__VizeOptionsBinding"),
        "{}",
        responses[2]
    );
}

#[test]
fn mistyped_options_are_reported_as_option_diagnostics() {
    let input = frames(&[
        initialize_request(),
        open_project_request(2, json!({ "optionsApi": "yes" }), json!({})),
    ]);
    let responses = exchange(&input);

    assert_eq!(
        responses[1]["result"]["optionDiagnostics"],
        json!([{
            "path": ["optionsApi"],
            "messageText": "Option 'optionsApi' requires a value of type boolean",
            "code": 3
        }])
    );
}

#[test]
fn non_object_options_are_reported_against_the_options_entry() {
    let input = frames(&[
        initialize_request(),
        open_project_request(2, json!(true), json!({})),
    ]);
    let responses = exchange(&input);

    assert!(responses[1].get("error").is_none());
    assert_eq!(
        responses[1]["result"]["optionDiagnostics"],
        json!([{
            "path": [],
            "messageText": "Content mapper options must be an object",
            "code": 1
        }])
    );
}

#[test]
fn transform_requires_an_opened_project_handle() {
    let input = frames(&[initialize_request(), transform_request(2, "<template />")]);
    let responses = exchange(&input);

    assert_eq!(responses[1]["error"]["code"], -32602);
    assert!(
        responses[1]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Unknown content mapper project")
    );
}

#[test]
fn close_project_releases_the_handle() {
    let input = frames(&[
        initialize_request(),
        open_project_request(2, Value::Null, json!({})),
        close_project_request(3),
        transform_request(4, "<template />"),
    ]);
    let responses = exchange(&input);

    assert!(responses[2]["result"].is_null());
    assert!(responses[2].get("error").is_none());
    assert_eq!(responses[3]["error"]["code"], -32602);
}

#[test]
fn reopening_a_handle_replaces_its_settings() {
    let source = r#"<script lang="ts">
export default { data() { return { count: 1 } } }
</script>
<template>{{ count }}</template>
"#;
    let input = frames(&[
        initialize_request(),
        open_project_request(2, json!({ "optionsApi": true }), json!({})),
        transform_request(3, source),
        open_project_request(4, json!({ "optionsApi": false }), json!({})),
        transform_request(5, source),
    ]);
    let responses = exchange(&input);

    assert!(
        responses[2]["result"]["text"]
            .as_str()
            .unwrap()
            .contains("__VizeOptionsBinding")
    );
    assert!(
        !responses[4]["result"]["text"]
            .as_str()
            .unwrap()
            .contains("__VizeOptionsBinding")
    );
}

const UNKNOWN_PROP_SOURCE: &str = r#"<script setup lang="ts">
import Child from './Child.vue';
</script>
<template><Child :count="1" :typo="2" /></template>"#;

fn configured_open(id: u8, handle: &str, path: &std::path::Path) -> Value {
    let mut request = open_project_request(id, Value::Null, json!({}));
    request["params"]["projectHandle"] = json!(handle);
    request["params"]["configFileName"] = json!(path);
    request
}

fn handled_transform(id: u8, handle: &str) -> Value {
    let mut request = transform_request(id, UNKNOWN_PROP_SOURCE);
    request["params"]["projectHandle"] = json!(handle);
    request
}

#[test]
fn host_selected_configs_keep_unknown_prop_checks_scoped_by_project_handle() {
    let directory = tempfile::tempdir().unwrap();
    let strict = directory.path().join("tsconfig.strict.json");
    let loose = directory.path().join("tsconfig.loose.json");
    std::fs::write(
        &strict,
        r#"{"vueCompilerOptions":{"checkUnknownProps":true}}"#,
    )
    .unwrap();
    std::fs::write(
        &loose,
        r#"{"vueCompilerOptions":{"strictTemplates":true,"checkUnknownProps":false}}"#,
    )
    .unwrap();
    let input = frames(&[
        initialize_request(),
        configured_open(2, "strict", &strict),
        configured_open(3, "loose", &loose),
        handled_transform(4, "strict"),
        handled_transform(5, "loose"),
        handled_transform(6, "strict"),
        configured_open(7, "strict", &loose),
        handled_transform(8, "strict"),
    ]);
    let responses = exchange(&input);
    let strict_code = responses[3]["result"]["text"].as_str().unwrap();
    let loose_code = responses[4]["result"]["text"].as_str().unwrap();
    let tail = |code: &str| {
        code.lines()
            .find(|line| line.contains("type __VizeComponentCheckTail"))
            .unwrap()
            .to_owned()
    };
    assert!(!tail(strict_code).contains("Record<string, unknown>"));
    assert!(tail(loose_code).contains("Record<string, unknown>"));
    assert_eq!(responses[3]["result"], responses[5]["result"]);
    assert_eq!(responses[4]["result"], responses[7]["result"]);
    assert_ne!(
        responses[1]["result"]["configIdentity"],
        responses[2]["result"]["configIdentity"]
    );
}

#[test]
fn reopening_after_inherited_config_change_replaces_the_transform_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    let base = directory.path().join("base.json");
    std::fs::write(&path, r#"{"extends":"./base.json"}"#).unwrap();
    std::fs::write(
        &base,
        r#"{"vueCompilerOptions":{"checkUnknownProps":true}}"#,
    )
    .unwrap();
    let first = exchange(&frames(&[
        initialize_request(),
        configured_open(2, "p1", &path),
        handled_transform(3, "p1"),
    ]));
    std::fs::write(
        &base,
        r#"{"vueCompilerOptions":{"checkUnknownProps":false}}"#,
    )
    .unwrap();
    let second = exchange(&frames(&[
        initialize_request(),
        configured_open(2, "p1", &path),
        handled_transform(3, "p1"),
    ]));
    assert_ne!(
        first[1]["result"]["configIdentity"],
        second[1]["result"]["configIdentity"]
    );
    assert_ne!(first[2]["result"]["text"], second[2]["result"]["text"]);
    assert_eq!(
        first[1]["result"]["watchedFiles"],
        second[1]["result"]["watchedFiles"]
    );
}

#[test]
fn malformed_inherited_config_does_not_open_a_default_settings_project() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    std::fs::write(&path, r#"{"extends":"./base.json"}"#).unwrap();
    std::fs::write(directory.path().join("base.json"), "{ bad JSON").unwrap();
    let good = directory.path().join("good.json");
    std::fs::write(&good, "{}").unwrap();
    let responses = exchange(&frames(&[
        initialize_request(),
        configured_open(2, "p1", &good),
        handled_transform(3, "p1"),
        configured_open(4, "p1", &path),
        handled_transform(5, "p1"),
    ]));
    assert!(responses[2]["result"]["text"].is_string());
    assert_eq!(responses[3]["error"]["code"], -32603);
    assert!(
        responses[3]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("base.json")
    );
    assert_eq!(responses[4]["error"]["code"], -32602);
}
