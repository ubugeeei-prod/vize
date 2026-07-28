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
    assert_eq!(responses[1]["result"]["scriptKind"], 3);
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
