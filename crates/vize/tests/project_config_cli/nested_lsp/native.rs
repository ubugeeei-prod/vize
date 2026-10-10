//! Native JSON-RPC evidence for project ownership and lifecycle refresh.
use super::super::{
    corsa_path, corsa_requirement,
    typecheck::{link_vue, workspace_root},
};
use super::{
    LspProcess, SOURCE, Value, cstr, file_uri, finish, fixture, initialize, json, settings, write,
};

pub(super) fn runtime() -> Option<String> {
    corsa_requirement::required_or_skip(corsa_path::resolve(workspace_root()))
}

pub(super) fn publication(
    lsp: &mut LspProcess,
    uri: &str,
    version: i64,
    expected: &Value,
) -> Value {
    let matches = |message: &Value| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri
            && message["params"]["version"] == version
            && message["params"]["diagnostics"] == *expected
    };
    let message = lsp
        .published_diagnostics()
        .into_iter()
        .find(matches)
        .unwrap_or_else(|| lsp.recv_matching(matches));
    assert_eq!(message["params"]["diagnostics"], *expected, "{message:#}");
    message
}

pub(super) fn mismatch(from: &str, to: &str, line: i64, variable: &str) -> Value {
    json!([{
        "range":{"start":{"line":line,"character":6},"end":{"line":line,"character":6+variable.len()}},
        "severity":1,"code":2322,"source":"vize/types",
        "message":cstr!("Type '{from}' is not assignable to type '{to}'.")
    }])
}

#[test]
fn nested_native_diagnostics_use_their_own_relative_tsconfig_and_alias_types() {
    let Some(runtime) = runtime() else {
        return;
    };
    let project = fixture(Some(&runtime));
    let root = project.path();
    link_vue(root);
    let mut lsp = initialize(root, json!({"hover":true}));
    for (id, name) in [(60, "a"), (61, "b")] {
        lsp.send(json!({"jsonrpc":"2.0","id":id,"method":"textDocument/hover","params":{
            "textDocument":{"uri":file_uri(&root.join(cstr!("packages/{name}/src/App.vue")))},"position":{"line":2,"character":7}
        }}));
    }
    for _ in 0..2 {
        let response = lsp.recv_matching(|message| message["id"] == 60 || message["id"] == 61);
        assert!(
            response["error"].is_null() && !response["result"].is_null(),
            "{response:#}"
        );
        let ty = if response["id"] == 60 {
            "number"
        } else {
            "string"
        };
        assert!(response["result"].to_string().contains(ty), "{response:#}");
    }
    for name in ["a", "b"] {
        let uri = file_uri(&root.join(cstr!("packages/{name}/src/App.vue")));
        publication(&mut lsp, &uri, 1, &json!([]));
    }
    for (name, from, to) in [("a", "number", "string"), ("b", "string", "number")] {
        let uri = file_uri(&root.join(cstr!("packages/{name}/src/App.vue")));
        lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
            "textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":SOURCE.replace("VALUE_TYPE",to)}]
        }}));
        publication(&mut lsp, &uri, 2, &mismatch(from, to, 2, "value"));
    }
    // A config change must clear A's current error without any document request.
    let config = root.join("packages/a/vite.config.mjs");
    std::fs::write(
        &config,
        settings(false, true, Some(&runtime)).replace("typecheck:true", "typecheck:false"),
    )
    .unwrap();
    lsp.send(
        json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{
            "changes":[{"uri":file_uri(&config),"type":2}]
        }}),
    );
    let a = file_uri(&root.join("packages/a/src/App.vue"));
    let cleared = lsp.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == a.as_str()
            && message["params"]["version"] == 2
            && message["params"]["diagnostics"] == json!([])
    });
    assert_eq!(cleared["params"]["diagnostics"], json!([]));
    finish(lsp);
}

#[test]
fn cross_package_unsaved_dependency_edit_and_close_refresh_the_importer() {
    let Some(runtime) = runtime() else {
        return;
    };
    let project = fixture(Some(&runtime));
    let root = project.path();
    link_vue(root);
    const SHARED: &str = "export const count = 1;\n";
    const CROSS: &str = "<script setup lang=\"ts\">\nimport { count } from '../../b/src/Shared';\nconst value: number = count;\n</script>\n<template>{{ value }}</template>\n";
    write(root, "packages/b/src/Shared.ts", SHARED);
    write(root, "packages/a/src/Cross.vue", CROSS);
    let mut lsp = initialize(root, Value::Null);
    let importer = file_uri(&root.join("packages/a/src/Cross.vue"));
    let dependency = file_uri(&root.join("packages/b/src/Shared.ts"));
    for (uri, language, text) in [
        (&importer, "vue", CROSS),
        (&dependency, "typescript", SHARED),
    ] {
        lsp.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":uri,"languageId":language,"version":1,"text":text}
            }}),
        );
    }
    publication(&mut lsp, &importer, 1, &json!([]));
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
        "textDocument":{"uri":dependency,"version":2},"contentChanges":[{"text":"export const count = 'wrong';\n"}]
    }}));
    let expected = mismatch("string", "number", 2, "value");
    publication(&mut lsp, &importer, 1, &expected);
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":dependency}}}));
    let restored = lsp.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == importer.as_str()
            && message["params"]["version"] == 1
            && message["params"]["diagnostics"] == json!([])
    });
    assert_eq!(restored["params"]["diagnostics"], json!([]));
    finish(lsp);
}
