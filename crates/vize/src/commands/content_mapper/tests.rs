use std::io::{BufReader, Cursor};

use serde_json::{Value, json};
use vize_carton::cstr;

use super::serve;

#[test]
fn negotiates_utf8_and_transforms_vue_sfc() {
    let input = frames(&[
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": 1,
                "positionEncodings": ["utf-16", "utf-8"],
                "locale": "en-US"
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "transform",
            "params": {
                "fileName": "/project/App.vue",
                "content": "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template>{{ count }}</template>\n",
                "compilerOptions": {}
            }
        }),
    ]);
    let responses = exchange(&input);

    assert_eq!(responses[0]["result"]["protocolVersion"], 1);
    assert_eq!(responses[0]["result"]["positionEncoding"], "utf-8");
    assert_eq!(responses[0]["result"]["diagnosticSource"], "vize");
    assert!(responses[1]["result"].get("scriptKind").is_none());
    assert!(
        responses[1]["result"]["text"]
            .as_str()
            .unwrap()
            .contains("const count = 1")
    );
    assert!(
        !responses[1]["result"]["mappings"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn parse_errors_are_successful_transform_results() {
    let input = frames(&[
        initialize_request(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "transform",
            "params": {
                "fileName": "Broken.vue",
                "content": "<template><div></template>",
                "compilerOptions": {}
            }
        }),
    ]);
    let responses = exchange(&input);

    assert!(responses[1].get("error").is_none());
    assert!(
        !responses[1]["result"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn rejects_transform_before_initialize() {
    let input = frames(&[json!({
        "jsonrpc": "2.0",
        "id": "early",
        "method": "transform",
        "params": {
            "fileName": "App.vue",
            "content": "<template />",
            "compilerOptions": {}
        }
    })]);
    let responses = exchange(&input);

    assert_eq!(responses[0]["id"], "early");
    assert_eq!(responses[0]["error"]["code"], -32002);
}

#[test]
fn transform_honors_static_options_api_setting() {
    let source = r#"<script lang="ts">
export default { data() { return { count: 1 } } }
</script>
<template>{{ count }}</template>
"#;
    let input = frames(&[
        initialize_request(),
        transform_request(2, source, json!({ "optionsApi": true })),
        transform_request(3, source, json!({ "optionsApi": false })),
    ]);
    let responses = exchange(&input);

    assert!(
        responses[1]["result"]["text"]
            .as_str()
            .unwrap()
            .contains("__VizeOptionsBinding")
    );
    assert!(
        !responses[2]["result"]["text"]
            .as_str()
            .unwrap()
            .contains("__VizeOptionsBinding")
    );
}

#[test]
fn transform_honors_no_unused_locals_compiler_option() {
    let source = r#"<script setup lang="ts">
const used = 1
const unused = 2
</script>
<template>{{ used }}</template>
"#;
    let input = frames(&[
        initialize_request(),
        transform_request_with_compiler_options(2, source, json!({ "noUnusedLocals": true })),
        transform_request_with_compiler_options(3, source, json!({ "noUnusedLocals": false })),
    ]);
    let responses = exchange(&input);
    let preserving = responses[1]["result"]["text"].as_str().unwrap();
    let suppressing = responses[2]["result"]["text"].as_str().unwrap();

    assert!(preserving.contains("void used;"), "{preserving}");
    assert!(!preserving.contains("void unused;"), "{preserving}");
    assert!(suppressing.contains("void used;"), "{suppressing}");
    assert!(suppressing.contains("void unused;"), "{suppressing}");
}

#[test]
fn transform_defaults_options_api_on_for_absent_null_and_empty_options() {
    let source = r#"<script lang="ts">
export default { data() { return { count: 1 } } }
</script>
<template>{{ count }}</template>
"#;
    let without_options = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "transform",
        "params": {
            "fileName": "Options.vue",
            "content": source,
            "compilerOptions": {}
        }
    });
    let input = frames(&[
        initialize_request(),
        without_options,
        transform_request(3, source, Value::Null),
        transform_request(4, source, json!({})),
    ]);
    let responses = exchange(&input);

    for response in &responses[1..] {
        assert!(
            response["result"]["text"]
                .as_str()
                .unwrap()
                .contains("__VizeOptionsBinding"),
            "expected default-on Options API output: {response}"
        );
    }
}

#[test]
fn rejects_unknown_transform_options() {
    let input = frames(&[
        initialize_request(),
        transform_request(2, "<template />", json!({ "optionApi": true })),
    ]);
    let responses = exchange(&input);

    assert_eq!(responses[1]["error"]["code"], -32602);
    assert!(
        responses[1]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("unknown field `optionApi`")
    );
}

#[test]
fn rejects_non_object_transform_options() {
    let input = frames(&[
        initialize_request(),
        transform_request(2, "<template />", json!(true)),
    ]);
    let responses = exchange(&input);

    assert_eq!(responses[1]["error"]["code"], -32602);
    assert!(
        responses[1]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("expected struct TransformOptions")
    );
}

fn initialize_request() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": 1,
            "positionEncodings": ["utf-8"]
        }
    })
}

fn transform_request(id: u8, content: &str, options: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "transform",
        "params": {
            "fileName": "Options.vue",
            "content": content,
            "options": options,
            "compilerOptions": {}
        }
    })
}

fn transform_request_with_compiler_options(
    id: u8,
    content: &str,
    compiler_options: Value,
) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "transform",
        "params": {
            "fileName": "Unused.vue",
            "content": content,
            "compilerOptions": compiler_options
        }
    })
}

fn frames(values: &[Value]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in values {
        let body = serde_json::to_vec(value).unwrap();
        bytes.extend_from_slice(cstr!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
        bytes.extend_from_slice(&body);
    }
    bytes
}

fn exchange(input: &[u8]) -> Vec<Value> {
    let mut reader = BufReader::new(Cursor::new(input));
    let mut output = Vec::new();
    serve(&mut reader, &mut output).unwrap();
    decode_frames(&output)
}

fn decode_frames(output: &[u8]) -> Vec<Value> {
    let mut reader = BufReader::new(Cursor::new(output));
    let mut values = Vec::new();
    while let Some(body) = super::read_frame(&mut reader).unwrap() {
        values.push(serde_json::from_slice(&body).unwrap());
    }
    values
}
