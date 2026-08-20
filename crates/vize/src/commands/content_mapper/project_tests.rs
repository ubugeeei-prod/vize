use serde_json::{Value, json};

use super::test_support::{
    close_project_request, exchange, frames, initialize_request, open_project_request,
    transform_request,
};

#[test]
fn open_project_returns_empty_config_identity_without_watched_files() {
    let input = frames(&[
        initialize_request(),
        open_project_request(2, json!({ "optionsApi": true }), json!({})),
    ]);
    let responses = exchange(&input);

    assert_eq!(responses[1]["result"]["configIdentity"], "");
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
