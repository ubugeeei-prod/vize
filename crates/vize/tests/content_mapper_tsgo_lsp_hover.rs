//! Component props, emits, slots and their JSDoc in hover through tsgo's own
//! LSP with the Vize content mapper (the editor backend without `vize lsp`).
#![expect(clippy::disallowed_methods, reason = "fixture URIs are std strings")]

use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::content_mapper_lsp_support::{install_packages, position, workspace_root};
use crate::lsp_component_api_hover_cli::{CASES, FILES};
use crate::support::lsp_process::{LspProcess, file_uri};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";

/// One request, answering the server's own requests while it is pending.
fn request(lsp: &mut LspProcess, id: i64, method: &str, params: Value) -> Value {
    lsp.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
    loop {
        let message = lsp.recv_matching(|message| {
            message.get("id").and_then(Value::as_i64) == Some(id)
                || message.get("method").is_some() && message.get("id").is_some()
        });
        let Some(server_method) = message.get("method").and_then(Value::as_str) else {
            assert!(message.get("error").is_none(), "{method}: {message:#}");
            return message.get("result").cloned().unwrap_or(Value::Null);
        };
        let result = if server_method == "workspace/configuration" {
            let count = message
                .pointer("/params/items")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            Value::Array(vec![Value::Null; count])
        } else {
            Value::Null
        };
        lsp.send(json!({ "jsonrpc": "2.0", "id": message.get("id"), "result": result }));
    }
}

#[test]
fn standard_tsgo_lsp_hovers_component_api_with_jsdoc() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping Content Mapper component API hover: {TSGO_ENV} is not set");
        return;
    };
    let expected: Value = serde_json::from_str(include_str!(
        "fixtures/component_api_hover/hover.tsgo-mapper.json"
    ))
    .unwrap();
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-lsp-hover-")
        .tempdir_in(cases_root)
        .unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/component_api_hover");
    std::fs::copy(
        fixture.join("tsconfig.json"),
        project.path().join("tsconfig.json"),
    )
    .unwrap();
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    let documents: Vec<_> = FILES
        .iter()
        .map(|(name, source)| {
            let path = project.path().join("src").join(name);
            std::fs::write(&path, source).unwrap();
            (*name, *source, file_uri(&path).to_string())
        })
        .collect();
    install_packages(project.path());

    let mut lsp = LspProcess::spawn_native(project.path(), &tsgo);
    let initialized = request(
        &mut lsp,
        1,
        "initialize",
        json!({
            "processId": null,
            "rootUri": file_uri(project.path()),
            "workspaceFolders": [{ "uri": file_uri(project.path()), "name": "content-mapper-hover" }],
            "capabilities": {
                "workspace": { "configuration": true, "workspaceFolders": true },
                "textDocument": {
                    "hover": { "contentFormat": ["markdown", "plaintext"] },
                    "diagnostic": { "dynamicRegistration": false }
                }
            },
            "initializationOptions": { "runExternalCode": true }
        }),
    );
    assert!(initialized["capabilities"].is_object(), "{initialized:#}");
    lsp.send(json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
    for (name, source, uri) in &documents {
        let language = if name.ends_with(".vue") {
            "vue"
        } else {
            "typescript"
        };
        lsp.send(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": { "uri": uri, "languageId": language, "version": 1, "text": source } }
        }));
    }
    let mut id = 2;
    // A pull loads the project and every mapped SFC before the first hover.
    for (name, _, uri) in &documents {
        let report = request(
            &mut lsp,
            id,
            "textDocument/diagnostic",
            json!({ "textDocument": { "uri": uri } }),
        );
        id += 1;
        assert_eq!(report["items"], json!([]), "{name}: {report:#}");
    }
    let mut actual = Map::new();
    for (case, file, needle) in CASES {
        let (_, source, uri) = documents.iter().find(|(name, _, _)| *name == file).unwrap();
        let offset = source.find(needle).expect("fixture needle");
        let hover = request(
            &mut lsp,
            id,
            "textDocument/hover",
            json!({ "textDocument": { "uri": uri }, "position": position(source, offset) }),
        );
        id += 1;
        actual.insert(case.into(), hover);
    }
    lsp.shutdown();
    let actual = Value::Object(actual);
    if actual != expected {
        // The whole packet, so a provider update can be reviewed and pinned at once.
        eprintln!("{}", serde_json::to_string_pretty(&actual).unwrap());
    }
    for (case, _, _) in CASES {
        assert_eq!(actual[case], expected[case], "{case}");
    }
    // tsgo trims and joins the hovers of every projection a position maps
    // to. A slot pattern mapped once (#12486) answers exactly like its use.
    assert_eq!(
        actual["slot prop pattern"]["contents"],
        actual["slot prop use"]["contents"],
    );
}
