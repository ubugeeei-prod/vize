//! Global editor requests and file notifications use current package policy.
use super::{
    LspProcess, SOURCE, Value, cstr, file_uri, finish, fixture, initialize, json, native, write,
};
use std::path::Path;

fn watch_root(lsp: &mut LspProcess, root: &Path) {
    let a = file_uri(&root.join("packages/a/src/App.vue"));
    let b = file_uri(&root.join("packages/b/src/App.vue"));
    native::publication(lsp, &a, 1, &json!([]));
    native::publication(lsp, &b, 1, &json!([]));
    write(
        root,
        "vite.config.mjs",
        "export default {vize:{lsp:{lint:false,typecheck:false,workspaceSymbols:true,fileRename:true}}};",
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[{"uri":file_uri(&root.join("vite.config.mjs")),"type":2}]}}));
    let (mut a_seen, mut b_seen) = (false, false);
    while !a_seen || !b_seen {
        let message = lsp.recv_matching(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && (message["params"]["uri"] == a.as_str()
                    || message["params"]["uri"] == b.as_str())
                && message["params"]["version"] == 1
        });
        a_seen |= message["params"]["uri"] == a.as_str();
        b_seen |= message["params"]["uri"] == b.as_str();
        assert_eq!(message["params"]["diagnostics"], json!([]), "{message:#}");
    }
}

#[test]
fn workspace_symbols_and_alias_file_edits_preserve_package_flags_after_primary_reload() {
    let project = fixture(None);
    let root = project.path();
    write(
        root,
        "vite.config.mjs",
        "export default {vize:{lsp:{lint:false,typecheck:false,workspaceSymbols:false,fileRename:false}}};",
    );
    let a = root.join("packages/a/vite.config.mjs");
    let authored = std::fs::read_to_string(&a).unwrap().replace(
        "lint:false",
        "workspaceSymbols:false,fileRename:false,lint:false",
    );
    std::fs::write(a, authored).unwrap();
    let mut lsp = initialize(root, Value::Null);
    watch_root(&mut lsp, root);
    let b = file_uri(&root.join("packages/b/src/App.vue"));
    lsp.send(
        json!({"jsonrpc":"2.0","id":50,"method":"workspace/symbol","params":{"query":"message"}}),
    );
    let symbols = lsp.recv_response(50);
    assert!(symbols["error"].is_null(), "{symbols:#}");
    assert_eq!(
        symbols["result"],
        json!([{"name":"message","kind":14,"location":{"uri":b,"range":{"start":{"line":3,"character":0},"end":{"line":3,"character":0}}},"containerName":"script setup"}]),
        "{symbols:#}"
    );
    let files = ["a", "b"].map(|name| json!({"oldUri":file_uri(&root.join(cstr!("packages/{name}/src/contracts.ts"))),"newUri":file_uri(&root.canonicalize().unwrap().join(cstr!("packages/{name}/src/updated.ts")))}));
    lsp.send(json!({"jsonrpc":"2.0","id":51,"method":"workspace/willRenameFiles","params":{"files":files}}));
    let edits = lsp.recv_response(51);
    assert!(edits["error"].is_null(), "{edits:#}");
    let line = SOURCE.lines().nth(1).unwrap();
    let start = line.find("@shared/contracts").unwrap();
    assert_eq!(
        edits["result"],
        json!({"changes":{b:[{"range":{"start":{"line":1,"character":start},"end":{"line":1,"character":start+"@shared/contracts".len()}},"newText":"@shared/updated"}]}}),
        "{edits:#}"
    );
    finish(lsp);
}

#[test]
fn explicit_bulk_provider_disables_survive_primary_reload_and_package_defaults() {
    let project = fixture(None);
    let root = project.path();
    let mut lsp = initialize(root, json!({"workspaceSymbols":false,"fileRename":false}));
    watch_root(&mut lsp, root);
    lsp.send(
        json!({"jsonrpc":"2.0","id":50,"method":"workspace/symbol","params":{"query":"message"}}),
    );
    let symbols = lsp.recv_response(50);
    assert!(symbols["error"].is_null(), "{symbols:#}");
    assert!(symbols["result"].is_null(), "{symbols:#}");
    lsp.send(json!({"jsonrpc":"2.0","id":51,"method":"workspace/willRenameFiles","params":{"files":[{"oldUri":file_uri(&root.join("packages/b/src/contracts.ts")),"newUri":file_uri(&root.join("packages/b/src/updated.ts"))}]}}));
    let edits = lsp.recv_response(51);
    assert!(edits["error"].is_null(), "{edits:#}");
    assert!(edits["result"].is_null(), "{edits:#}");
    finish(lsp);
}

fn next_diagnostics(lsp: &mut LspProcess, uri: &str) -> Value {
    let message = lsp.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri
            && message["params"]["version"] == 1
    });
    message["params"]["diagnostics"].clone()
}

#[test]
fn cross_package_file_rename_delete_and_create_refresh_native_buffers_after_primary_reload() {
    let Some(runtime) = native::runtime() else {
        return;
    };
    let project = fixture(Some(&runtime));
    let root = project.path().canonicalize().unwrap();
    super::super::typecheck::link_vue(&root);
    write(
        &root,
        "vite.config.mjs",
        "export default {vize:{lsp:{lint:false,typecheck:false}}};",
    );
    let mut lsp = initialize(&root, Value::Null);
    for name in ["a", "b"] {
        native::publication(
            &mut lsp,
            &file_uri(&root.join(cstr!("packages/{name}/src/App.vue"))),
            1,
            &json!([]),
        );
    }
    watch_root(&mut lsp, &root);
    let old = root.join("packages/a/src/Move.vue");
    let new = root.join("packages/b/src/Moved.vue");
    let source = SOURCE.replace("VALUE_TYPE", "number");
    write(&root, "packages/a/src/Move.vue", &source);
    let old_uri = file_uri(&old);
    let new_uri = file_uri(&new);
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":old_uri,"languageId":"vue","version":1,"text":source}}}));
    native::publication(&mut lsp, &old_uri, 1, &json!([]));
    std::fs::rename(&old, &new).unwrap();
    lsp.send(json!({"jsonrpc":"2.0","method":"workspace/didRenameFiles","params":{"files":[{"oldUri":old_uri,"newUri":new_uri}]}}));
    assert_eq!(
        next_diagnostics(&mut lsp, &new_uri),
        native::mismatch("string", "number", 2, "value")
    );
    let cleared = lsp
        .published_diagnostics()
        .into_iter()
        .find(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == old_uri.as_str()
                && message["params"].get("version").is_none()
        })
        .expect("rename must clear the old authored URI");
    assert_eq!(cleared["params"]["diagnostics"], json!([]));
    let contracts = root.join("packages/b/src/contracts.ts");
    let contracts_uri = file_uri(&contracts);
    std::fs::remove_file(&contracts).unwrap();
    lsp.send(json!({"jsonrpc":"2.0","method":"workspace/didDeleteFiles","params":{"files":[{"uri":contracts_uri}]}}));
    let missing = json!([{"range":{"start":{"line":1,"character":22},"end":{"line":1,"character":41}},"severity":1,"code":2307,"source":"vize/types","message":"Cannot find module '@shared/contracts' or its corresponding type declarations."}]);
    assert_eq!(next_diagnostics(&mut lsp, &new_uri), missing);
    write(
        &root,
        "packages/b/src/contracts.ts",
        "export const count = 1;\n",
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"workspace/didCreateFiles","params":{"files":[{"uri":contracts_uri}]}}));
    assert_eq!(next_diagnostics(&mut lsp, &new_uri), json!([]));
    let b = file_uri(&root.join("packages/b/src/App.vue"));
    native::publication(
        &mut lsp,
        &b,
        1,
        &native::mismatch("number", "string", 2, "value"),
    );
    finish(lsp);
}
