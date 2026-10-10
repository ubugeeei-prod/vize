//! One editor workspace must not make sibling package settings order-dependent.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "public JSON-RPC fixtures use protocol strings"
)]
use super::{
    lsp_process::{LspProcess, file_uri},
    support::{assert_no_dedicated_config, write},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
use vize_l0::cstr;

const SOURCE: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/nested-project-settings-8371/App.vue.txt"
);
const TSCONFIG: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/nested-project-settings-8371/tsconfig.json.txt"
);

fn settings(hover: bool, single_quote: bool, runtime: Option<&str>) -> String {
    cstr!(
        "import {{appendFileSync}} from 'node:fs'; appendFileSync(new URL('./config-evaluations.txt', import.meta.url), '1'); export default {{vize:{{formatter:{{singleQuote:{single_quote}}},typeChecker:{{tsconfig:'config/tsconfig.app.json',runtimePath:{}}},lsp:{{hover:{hover},lint:false,typecheck:{}}}}}}};",
        serde_json::to_string(&runtime).unwrap(),
        runtime.is_some()
    ).into()
}

fn fixture(runtime: Option<&str>) -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    write(
        root,
        "package.json",
        r#"{"private":true,"workspaces":["packages/*"]}"#,
    );
    write(
        root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"packages/a/config/tsconfig.app.json"},{"path":"packages/b/config/tsconfig.app.json"}]}"#,
    );
    for (name, hover, quotes, ty, contract) in [
        ("a", false, true, "number", "1"),
        ("b", true, false, "string", "'one'"),
    ] {
        let package = root.join("packages").join(name);
        write(&package, "package.json", "{}");
        let settings = settings(hover, quotes, runtime);
        let settings = if name == "a" {
            settings
                .replace("export default {vize:", "export default {root:'src',vize:")
                .replace("tsconfig:'config/", "tsconfig:'../config/")
        } else {
            settings
        };
        write(&package, "vite.config.mjs", &settings);
        write(&package, "config/tsconfig.app.json", TSCONFIG);
        write(&package, "src/App.vue", &SOURCE.replace("VALUE_TYPE", ty));
        write(
            &package,
            "src/contracts.ts",
            &cstr!("export const count = {contract};"),
        );
        assert_no_dedicated_config(&package);
    }
    project
}

fn initialize(root: &Path, options: Value) -> LspProcess {
    let expected_formatting = options.get("formatting").and_then(Value::as_bool);
    let mut lsp = LspProcess::spawn(root);
    lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "processId":null,"rootUri":file_uri(root),"capabilities":{},"initializationOptions":options
    }}));
    let response = lsp.recv_response(1);
    assert!(response["error"].is_null(), "{response:#}");
    assert_eq!(
        response["result"]["capabilities"]["documentFormattingProvider"]
            .as_bool()
            .unwrap_or(false),
        expected_formatting.unwrap_or(true),
        "{response:#}"
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    for (name, ty) in [("a", "number"), ("b", "string")] {
        lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
            "textDocument":{"uri":file_uri(&root.join(cstr!("packages/{name}/src/App.vue"))),"languageId":"vue","version":1,"text":SOURCE.replace("VALUE_TYPE",ty)}
        }}));
    }
    lsp
}

fn finish(mut lsp: LspProcess) {
    lsp.send(json!({"jsonrpc":"2.0","id":99,"method":"shutdown"}));
    assert!(lsp.recv_response(99)["result"].is_null());
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(lsp.wait_for_exit().success());
}

fn concurrent_format(lsp: &mut LspProcess, root: &Path, first_id: i64) -> BTreeMap<i64, Value> {
    for (offset, name) in [(0, "a"), (1, "b"), (2, "a"), (3, "b")] {
        lsp.send(json!({"jsonrpc":"2.0","id":first_id+offset,"method":"textDocument/formatting","params":{
            "textDocument":{"uri":file_uri(&root.join(cstr!("packages/{name}/src/App.vue")))},"options":{"tabSize":2,"insertSpaces":true}
        }}));
    }
    (0..4)
        .map(|_| {
            let response = lsp.recv_matching(|message| {
                message["id"]
                    .as_i64()
                    .is_some_and(|id| (first_id..first_id + 4).contains(&id))
            });
            assert!(response["error"].is_null(), "{response:#}");
            (response["id"].as_i64().unwrap(), response["result"].clone())
        })
        .collect()
}

fn assert_quotes(responses: &BTreeMap<i64, Value>, first_id: i64, quotes: [&str; 2]) {
    for offset in 0..4 {
        let response = &responses[&(first_id + offset)];
        let formatted = response[0]["newText"]
            .as_str()
            .expect("formatting must be enabled");
        assert!(
            formatted.contains(quotes[offset as usize % 2]),
            "{response:#}"
        );
    }
}

#[test]
fn nested_settings_are_stable_under_concurrent_requests_and_watch_reload() {
    let project = fixture(None);
    let root = project.path();
    let cold = std::time::Instant::now();
    let mut lsp = initialize(root, Value::Null);
    let responses = concurrent_format(&mut lsp, root, 10);
    assert_quotes(
        &responses,
        10,
        ["const message = 'hello'", "const message = \"hello\""],
    );
    let cold_elapsed = cold.elapsed();
    let evaluations = ["a", "b"].map(|name| {
        std::fs::read_to_string(root.join(cstr!("packages/{name}/config-evaluations.txt"))).unwrap()
    });
    let warm = std::time::Instant::now();
    // Both request orders retain their own package settings, with no reimport.
    assert_quotes(
        &concurrent_format(&mut lsp, root, 20),
        20,
        ["const message = 'hello'", "const message = \"hello\""],
    );
    eprintln!(
        "nested LSP cold open + four formatting requests: {cold_elapsed:?}; warm four requests: {:?}",
        warm.elapsed()
    );
    for (name, evaluations) in ["a", "b"].into_iter().zip(evaluations) {
        assert_eq!(
            std::fs::read_to_string(root.join(cstr!("packages/{name}/config-evaluations.txt")))
                .unwrap(),
            evaluations
        );
    }
    let config = root.join("packages/a/vite.config.mjs");
    std::fs::write(&config, settings(false, false, None)).unwrap();
    lsp.send(
        json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{
            "changes":[{"uri":file_uri(&config),"type":2}]
        }}),
    );
    assert_quotes(
        &concurrent_format(&mut lsp, root, 30),
        30,
        ["const message = \"hello\"", "const message = \"hello\""],
    );
    std::fs::write(
        &config,
        settings(true, false, None).replace("lint:false", "formatting:false,lint:false"),
    )
    .unwrap();
    lsp.send(
        json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{
            "changes":[{"uri":file_uri(&config),"type":2}]
        }}),
    );
    let responses = concurrent_format(&mut lsp, root, 40);
    for (id, result) in responses {
        assert_eq!(result.is_null(), id % 2 == 0, "{id}: {result:#}");
    }
    finish(lsp);
}

#[test]
fn explicit_initialization_flags_win_over_each_nested_project() {
    let project = fixture(None);
    let root = project.path();
    let mut lsp = initialize(root, json!({"formatting":false,"hover":false}));
    for response in concurrent_format(&mut lsp, root, 10).values() {
        assert!(response.is_null(), "{response:#}");
    }
    for (id, name) in [(60, "a"), (61, "b")] {
        lsp.send(json!({"jsonrpc":"2.0","id":id,"method":"textDocument/hover","params":{
            "textDocument":{"uri":file_uri(&root.join(cstr!("packages/{name}/src/App.vue")))},"position":{"line":2,"character":7}
        }}));
        let response = lsp.recv_response(id);
        assert!(
            response["error"].is_null() && response["result"].is_null(),
            "{response:#}"
        );
    }
    finish(lsp);
}

#[path = "nested_lsp/ignores.rs"]
mod ignores;
#[path = "nested_lsp/native.rs"]
mod native;
