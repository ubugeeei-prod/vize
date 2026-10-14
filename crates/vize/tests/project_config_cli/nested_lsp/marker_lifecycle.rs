//! New and removed package boundaries refresh already-open authored buffers.
use super::{LspProcess, assert_no_dedicated_config, cstr, file_uri, finish, json, native, write};

#[test]
fn newly_created_and_removed_nested_config_refreshes_without_document_requests() {
    let Some(runtime) = native::runtime() else {
        return;
    };
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    super::super::typecheck::link_vue(root);
    write(root, "package.json", "{}");
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"Bundler"},"include":["**/*.ts","**/*.vue"]}"#,
    );
    write(
        root,
        "vite.config.mjs",
        &cstr!(
            "export default {{vize:{{typeChecker:{{runtimePath:{}}},lsp:{{lint:false,typecheck:true}}}}}};",
            serde_json::to_string(&runtime).unwrap()
        ),
    );
    const BAD: &str = include_str!(
        "../../../../../tests/_fixtures/differential/config/nested-project-settings-8371/Marker.vue.txt"
    );
    write(root, "nested/App.vue", BAD);
    let uri = file_uri(&root.join("nested/App.vue"));
    let config = root.join("nested/vite.config.mjs");
    let mut lsp = LspProcess::spawn(root);
    lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":file_uri(root),"capabilities":{}}}));
    assert!(lsp.recv_response(1)["error"].is_null());
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"vue","version":1,"text":BAD}}}));
    native::publication(
        &mut lsp,
        &uri,
        1,
        &native::mismatch("string", "number", 1, "value"),
    );
    write(
        root,
        "nested/vite.config.mjs",
        "export default {vize:{lsp:{lint:false,typecheck:false}}};",
    );
    let config_uri = file_uri(&config);
    lsp.send(json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":config_uri,"type":1}]}}));
    let cleared = lsp.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri.as_str()
            && message["params"]["version"] == 1
    });
    assert_eq!(cleared["params"]["diagnostics"], json!([]), "{cleared:#}");
    // The buffer remains owned by the new nested context until the marker is
    // removed. Restoration must use its latest unsaved text in the parent.
    const EDITED: &str = include_str!(
        "../../../../../tests/_fixtures/differential/config/nested-project-settings-8371/MarkerEdited.vue.txt"
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":EDITED}]}}));
    native::publication(&mut lsp, &uri, 2, &json!([]));
    std::fs::remove_file(&config).unwrap();
    lsp.send(json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":config_uri,"type":3}]}}));
    let restored = lsp.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri.as_str()
            && message["params"]["version"] == 2
    });
    assert_eq!(
        restored["params"]["diagnostics"],
        native::mismatch("number", "string", 1, "value"),
        "{restored:#}"
    );
    assert_no_dedicated_config(root);
    finish(lsp);
}
