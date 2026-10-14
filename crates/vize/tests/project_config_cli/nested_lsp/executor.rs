//! A cold package import must yield the real editor transport to warm siblings.
use super::{LspProcess, SOURCE, file_uri, finish, fixture, format_oracle, json, native, write};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

struct Release(PathBuf);
impl Drop for Release {
    fn drop(&mut self) {
        // Also release the Node child if a protocol assertion unwinds.
        std::fs::write(&self.0, "release").unwrap();
    }
}

#[test]
fn delayed_cold_package_import_does_not_block_warm_sibling_completion() {
    let project = fixture(None);
    let root = project.path();
    let config = root.join("packages/a/vite.config.mjs");
    let original = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, original.replace("export default", "import {existsSync,writeFileSync} from 'node:fs'; writeFileSync(new URL('./import-started', import.meta.url),'started'); while(!existsSync(new URL('./release-import', import.meta.url))) {await new Promise(resolve=>setTimeout(resolve,10));} export default")).unwrap();
    let release = Release(root.join("packages/a/release-import"));
    let started = root.join("packages/a/import-started");
    const SOURCE_B: &str = "<template><sp /></template>\n";
    write(root, "packages/b/src/App.vue", SOURCE_B);
    let a = file_uri(&root.join("packages/a/src/App.vue"));
    let b = file_uri(&root.join("packages/b/src/App.vue"));
    let mut lsp = LspProcess::spawn(root);
    lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":file_uri(root),"capabilities":{}}}));
    assert!(lsp.recv_response(1)["error"].is_null());
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":b,"languageId":"vue","version":1,"text":SOURCE_B}}}));
    let completion = |id| json!({"jsonrpc":"2.0","id":id,"method":"textDocument/completion","params":{"textDocument":{"uri":b},"position":{"line":0,"character":13}}});
    lsp.send(completion(2));
    let baseline = lsp.recv_response(2);
    let expected = json!([{"detail":"Native element","insertTextFormat":1,"kind":14,"label":"span","sortText":"span","textEdit":{"newText":"span","range":{"start":{"line":0,"character":11},"end":{"line":0,"character":13}}}}]);
    assert_eq!(baseline, json!({"jsonrpc":"2.0","id":2,"result":expected}));
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":a,"languageId":"vue","version":1,"text":SOURCE.replace("VALUE_TYPE","number")}}}));
    let deadline = Instant::now() + Duration::from_secs(15);
    while !started.exists() {
        assert!(
            Instant::now() < deadline,
            "cold config import did not start"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!release.0.exists());
    lsp.send(completion(3));
    let result = lsp.recv_response(3);
    assert!(!release.0.exists(), "warm completion must precede release");
    assert_eq!(result, json!({"jsonrpc":"2.0","id":3,"result":expected}));
    let changed = SOURCE.replace("VALUE_TYPE", "string");
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":a,"version":2},"contentChanges":[{"text":changed}]}}));
    for id in [4, 5] {
        lsp.send(json!({"jsonrpc":"2.0","id":id,"method":"textDocument/formatting","params":{"textDocument":{"uri":a},"options":{"tabSize":2,"insertSpaces":true}}}));
    }
    drop(release);
    let mut ids = std::collections::BTreeSet::new();
    for _ in 0..2 {
        let response = lsp
            .recv_matching(|message| message["id"].as_i64().is_some_and(|id| id == 4 || id == 5));
        ids.insert(response["id"].as_i64().unwrap());
        assert!(response["error"].is_null(), "{response:#}");
        assert_eq!(
            format_oracle::apply(&changed, &response["result"]),
            format_oracle::expected("string", "'")
        );
    }
    assert_eq!(ids, [4, 5].into_iter().collect());
    native::publication(&mut lsp, &a, 2, &json!([]));
    for name in ["a", "b"] {
        assert_eq!(
            std::fs::read_to_string(
                root.join(super::cstr!("packages/{name}/config-evaluations.txt"))
            )
            .unwrap(),
            "1"
        );
    }
    finish(lsp);
}

#[test]
fn shutdown_retires_pending_config_without_waiting_for_node_import() {
    let project = fixture(None);
    let root = project.path();
    let config = root.join("packages/a/vite.config.mjs");
    let original = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, original.replace("export default", "import {existsSync,writeFileSync} from 'node:fs'; writeFileSync(new URL('./import-started', import.meta.url),'started'); while(!existsSync(new URL('./release-import', import.meta.url))) {await new Promise(resolve=>setTimeout(resolve,10));} export default")).unwrap();
    let release = Release(root.join("packages/a/release-import"));
    let started = root.join("packages/a/import-started");
    let mut lsp = LspProcess::spawn(root);
    lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":file_uri(root),"capabilities":{}}}));
    assert!(lsp.recv_response(1)["error"].is_null());
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":file_uri(&root.join("packages/a/src/App.vue")),"languageId":"vue","version":1,"text":SOURCE.replace("VALUE_TYPE","number")}}}));
    let deadline = Instant::now() + Duration::from_secs(15);
    while !started.exists() {
        assert!(
            Instant::now() < deadline,
            "cold config import did not start"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    lsp.send(json!({"jsonrpc":"2.0","id":99,"method":"shutdown"}));
    assert_eq!(
        lsp.recv_response(99),
        json!({"jsonrpc":"2.0","id":99,"result":null})
    );
    assert!(!release.0.exists(), "shutdown must precede import release");
    drop(release);
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(lsp.wait_for_exit().success());
}

#[test]
fn rename_into_cold_package_keeps_later_destination_changes() {
    let project = fixture(None);
    let root = project.path();
    let config = root.join("packages/a/vite.config.mjs");
    let original = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, original.replace("export default", "import {existsSync,writeFileSync} from 'node:fs'; writeFileSync(new URL('./import-started', import.meta.url),'started'); while(!existsSync(new URL('./release-import', import.meta.url))) {await new Promise(resolve=>setTimeout(resolve,10));} export default")).unwrap();
    let release = Release(root.join("packages/a/release-import"));
    let started = root.join("packages/a/import-started");
    let old = root.join("packages/b/src/App.vue");
    let new = root.join("packages/a/src/Moved.vue");
    let old_uri = file_uri(&old);
    let new_uri = file_uri(&new);
    let sibling = root.join("packages/b/src/Sibling.vue");
    const SIBLING: &str = "<template><sp /></template>\n";
    write(root, "packages/b/src/Sibling.vue", SIBLING);
    let sibling_uri = file_uri(&sibling);
    let mut lsp = LspProcess::spawn(root);
    lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":file_uri(root),"capabilities":{}}}));
    assert!(lsp.recv_response(1)["error"].is_null());
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    let source = SOURCE.replace("VALUE_TYPE", "string");
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":old_uri,"languageId":"vue","version":1,"text":source}}}));
    native::publication(&mut lsp, &old_uri, 1, &json!([]));
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":sibling_uri,"languageId":"vue","version":1,"text":SIBLING}}}));
    std::fs::rename(&old, &new).unwrap();
    lsp.send(json!({"jsonrpc":"2.0","method":"workspace/didRenameFiles","params":{"files":[{"oldUri":old_uri,"newUri":new_uri}]}}));
    let deadline = Instant::now() + Duration::from_secs(15);
    while !started.exists() {
        assert!(
            Instant::now() < deadline,
            "destination import did not start"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let changed = SOURCE.replace("VALUE_TYPE", "number");
    lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":new_uri,"version":2},"contentChanges":[{"text":changed}]}}));
    lsp.send(json!({"jsonrpc":"2.0","id":3,"method":"textDocument/completion","params":{"textDocument":{"uri":sibling_uri},"position":{"line":0,"character":13}}}));
    assert_eq!(
        lsp.recv_response(3),
        json!({"jsonrpc":"2.0","id":3,"result":[{"detail":"Native element","insertTextFormat":1,"kind":14,"label":"span","sortText":"span","textEdit":{"newText":"span","range":{"start":{"line":0,"character":11},"end":{"line":0,"character":13}}}}]})
    );
    assert!(
        !release.0.exists(),
        "sibling must reply during pending rename"
    );
    lsp.send(json!({"jsonrpc":"2.0","id":2,"method":"textDocument/formatting","params":{"textDocument":{"uri":new_uri},"options":{"tabSize":2,"insertSpaces":true}}}));
    drop(release);
    let response = lsp.recv_response(2);
    assert!(response["error"].is_null(), "{response:#}");
    assert_eq!(
        format_oracle::apply(&changed, &response["result"]),
        format_oracle::expected("number", "'")
    );
    native::publication(&mut lsp, &new_uri, 2, &json!([]));
    let cleared = lsp
        .published_diagnostics()
        .into_iter()
        .find(|message| {
            message["params"]["uri"] == old_uri.as_str()
                && message["params"].get("version").is_none()
        })
        .expect("rename must clear its old URI");
    assert_eq!(
        cleared,
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":old_uri,"diagnostics":[]}})
    );
    finish(lsp);
}
