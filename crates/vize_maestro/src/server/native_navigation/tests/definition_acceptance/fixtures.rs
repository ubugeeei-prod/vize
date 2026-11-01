//! Original authored inputs and independent literal response locations.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use vize_l0::{CompactString, cstr};

pub(super) const URI: &str = "file:///artifact-definition-acceptance.vue";
pub(super) const OTHER: &str = "file:///lottie.vue";
pub(super) const AUTHORED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/script-cjk-navigation/App.vue.txt"
));
pub(super) const CHANGED: &str =
    "<script setup>/*😀*/ const café=1;</script>\r\n<template>{{café}}</template>";
pub(super) const CROSS_FILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/registered-lowercase-component-alias/App.vue.txt"
));
pub(super) const OTHER_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/registered-lowercase-component-alias/lottie.vue.txt"
));

pub(super) fn hash(bytes: &[u8]) -> CompactString {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect()
}

pub(super) fn initialize() -> Value {
    json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "capabilities":{},"initializationOptions":{
            "lint":false,"typecheck":false,"ecosystem":false
        }
    }})
}

pub(super) fn open(uri: &str, text: &str, version: i32) -> Value {
    json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
        "textDocument":{"uri":uri,"languageId":"vue","version":version,"text":text}
    }})
}

pub(super) fn definition(id: i32, line: u32, character: u32) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"vize/nativeDefinition","params":{
        "textDocument":{"uri":URI},"position":{"line":line,"character":character}
    }})
}

pub(super) fn target(line: u32, start: u32, end: u32) -> Value {
    json!({"uri":URI,"range":{
        "start":{"line":line,"character":start},
        "end":{"line":line,"character":end}
    }})
}

pub(super) fn result(id: i32, value: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":value})
}

pub(super) fn error(id: i32, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

pub(super) fn initialized_log() -> Value {
    json!({"jsonrpc":"2.0","method":"window/logMessage",
        "params":{"type":3,"message":"vize_maestro LSP server initialized"}})
}

pub(super) fn diagnostics(uri: &str, version: Option<i32>) -> Value {
    match version {
        Some(version) => json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
            "params":{"uri":uri,"diagnostics":[],"version":version}}),
        None => json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
            "params":{"uri":uri,"diagnostics":[]}}),
    }
}

pub(super) fn inputs() -> Value {
    json!({
        "authored":{"bytes":AUTHORED.len(),"sha256":hash(AUTHORED.as_bytes()),"source":AUTHORED},
        "changed":{"bytes":CHANGED.len(),"sha256":hash(CHANGED.as_bytes()),"source":CHANGED},
        "crossFile":{"bytes":CROSS_FILE.len(),"sha256":hash(CROSS_FILE.as_bytes()),"source":CROSS_FILE},
        "other":{"bytes":OTHER_SOURCE.len(),"sha256":hash(OTHER_SOURCE.as_bytes()),"source":OTHER_SOURCE}
    })
}

pub(super) fn compiled_sources() -> Value {
    let sources: [(&str, &[u8]); 6] = [
        (
            "source_project/navigation.rs",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/source_project/navigation.rs"
            )),
        ),
        (
            "source_project/navigation/definition_acceptance.rs",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/source_project/navigation/definition_acceptance.rs"
            )),
        ),
        (
            "server/native_navigation/tests.rs",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/server/native_navigation/tests.rs"
            )),
        ),
        (
            "server/native_navigation/tests/definition_acceptance.rs",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/server/native_navigation/tests/definition_acceptance.rs"
            )),
        ),
        (
            "server/native_navigation/tests/definition_acceptance/wire.rs",
            include_bytes!("wire.rs"),
        ),
        (
            "server/native_navigation/tests/definition_acceptance/fixtures.rs",
            include_bytes!("fixtures.rs"),
        ),
    ];
    Value::Array(
        sources
            .iter()
            .map(|(path, bytes)| {
                json!({
                    "path":path,"bytes":bytes.len(),"sha256":hash(bytes)
                })
            })
            .collect(),
    )
}
