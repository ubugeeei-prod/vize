//! Shared helpers for the content-mapper server tests.

use std::io::{BufReader, Cursor};

use serde_json::{Value, json};
use vize_s0::cstr;

pub(super) const PROJECT_HANDLE: &str = "p1";

pub(super) fn initialize_request() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "positionEncodings": ["utf-8"]
        }
    })
}

pub(super) fn open_project_request(id: u8, options: Value, compiler_options: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "openProject",
        "params": {
            "configFileName": "/project/tsconfig.json",
            "projectHandle": PROJECT_HANDLE,
            "options": options,
            "compilerOptions": compiler_options
        }
    })
}

pub(super) fn close_project_request(id: u8) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "closeProject",
        "params": { "projectHandle": PROJECT_HANDLE }
    })
}

pub(super) fn transform_request(id: u8, content: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "transform",
        "params": {
            "fileName": "/project/Options.vue",
            "content": content,
            "projectHandle": PROJECT_HANDLE
        }
    })
}

pub(super) fn frames(values: &[Value]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in values {
        let body = serde_json::to_vec(value).unwrap();
        bytes.extend_from_slice(cstr!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
        bytes.extend_from_slice(&body);
    }
    bytes
}

pub(super) fn exchange(input: &[u8]) -> Vec<Value> {
    let mut reader = BufReader::new(Cursor::new(input));
    let mut output = Vec::new();
    super::serve(&mut reader, &mut output).unwrap();
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
