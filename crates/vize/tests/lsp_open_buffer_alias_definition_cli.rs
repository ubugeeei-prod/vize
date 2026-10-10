#![cfg(feature = "maestro")]
#![expect(
    clippy::disallowed_types,
    reason = "fixture JSON and process boundaries use std strings"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "fixture inputs are owned std strings"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "test failures include authored phase names"
)]

use std::{fs, path::Path};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::Url;

#[path = "support/lsp_process.rs"]
mod lsp_process;
use lsp_process::LspProcess;

const CORPUS: &str = "../../tests/_fixtures/differential/lsp/open-buffer-alias-definition";

#[test]
fn alias_component_definitions_keep_complete_lf_lifecycle_contracts() {
    assert_corpus("\n");
}

#[test]
fn alias_component_definitions_keep_complete_crlf_lifecycle_contracts() {
    assert_corpus("\r\n");
}

fn assert_corpus(newline: &str) {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS);
    let manifest: Value =
        serde_json::from_slice(&fs::read(corpus.join("case.json")).unwrap()).unwrap();
    for file in manifest["files"].as_array().unwrap() {
        let bytes = fs::read(corpus.join(file["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            file["sha256"].as_str().unwrap()
        );
    }
    assert_contracts(&corpus, &manifest, newline);
}

fn assert_contracts(corpus: &Path, manifest: &Value, newline: &str) {
    let temp = tempfile::Builder::new()
        .prefix("vize-alias-子 [赤] #-")
        .tempdir()
        .unwrap();
    let root = temp.path().canonicalize().unwrap();
    let input = |name: &str| {
        fs::read_to_string(corpus.join(name))
            .unwrap()
            .replace('\n', newline)
    };
    fs::write(root.join("tsconfig.json"), input("tsconfig.json.txt")).unwrap();
    fs::write(root.join("vize.config.json"), input("vize.config.json.txt")).unwrap();
    let target = root.join(manifest["targetRuntimePath"].as_str().unwrap());
    let preferred = root.join(manifest["preferredRuntimePath"].as_str().unwrap());
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::create_dir_all(preferred.parent().unwrap()).unwrap();
    let target_uri = Url::from_file_path(&target).unwrap();
    let preferred_uri = Url::from_file_path(&preferred).unwrap();
    let root_uri = Url::from_directory_path(&root).unwrap();
    assert_eq!(
        target_uri.as_str(),
        format!("{root_uri}components/%E5%AD%90%20[%E8%B5%A4]%20%23.vue")
    );
    assert_eq!(
        preferred_uri.as_str(),
        format!("{root_uri}preferred/%E5%AD%90%20[%E8%B5%A4]%20%23.vue")
    );
    assert!(target_uri.fragment().is_none());
    let expected: Value = serde_json::from_str(
        &input("responses.expected.json")
            .replace("$COMPONENT_URI", target_uri.as_str())
            .replace("$PREFERRED_URI", preferred_uri.as_str()),
    )
    .unwrap();
    let mut server = LspProcess::spawn(&root);
    server.retain_protocol();
    server.send(
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "processId":null,"rootUri":Url::from_file_path(&root).unwrap(),"capabilities":{},
            "initializationOptions":{"editor":true,"definition":true,"lint":false,"typecheck":false}
        }}),
    );
    let initialized = server.recv_response(1);
    assert!(initialized.get("error").is_none(), "{initialized}");
    server.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    let relative = Url::from_file_path(root.join("RelativeParent.vue")).unwrap();
    let alias = Url::from_file_path(root.join("AliasParent.vue")).unwrap();
    let fallback = Url::from_file_path(root.join("FallbackParent.vue")).unwrap();
    for (uri, name) in [
        (&relative, "RelativeParent.vue.txt"),
        (&alias, "AliasParent.vue.txt"),
        (&fallback, "FallbackParent.vue.txt"),
    ] {
        fs::write(uri.to_file_path().unwrap(), input(name)).unwrap();
        open(&mut server, uri, &input(name));
    }
    let mut id = 2;
    let mut observations = Vec::new();
    let mut query = |server: &mut LspProcess, uri: &Url, phase: &str| {
        server.send(
            json!({"jsonrpc":"2.0","id":id,"method":"textDocument/definition","params":{
                "textDocument":{"uri":uri},"position":manifest["queryPosition"]
            }}),
        );
        let wanted = json!({"jsonrpc":"2.0","id":id,"result":expected[phase]});
        let actual = server.recv_response(id);
        observations.push(json!({"phase":phase,"expected":wanted,"actual":actual}));
        id += 1;
    };
    assert!(!target.exists());
    query(&mut server, &relative, "relativeAbsent");
    query(&mut server, &alias, "aliasAbsent");
    open(&mut server, &target_uri, &input("Child.vue.txt"));
    assert!(
        !target.exists(),
        "didOpen must not create the dependency on disk"
    );
    query(&mut server, &relative, "relativeOpen");
    query(&mut server, &alias, "aliasOpen");
    close(&mut server, &target_uri);
    query(&mut server, &relative, "relativeClosed");
    query(&mut server, &alias, "aliasClosed");
    fs::write(&target, input("Child.vue.txt")).unwrap();
    query(&mut server, &relative, "relativeSaved");
    query(&mut server, &alias, "aliasSaved");
    fs::remove_file(&target).unwrap();
    query(&mut server, &relative, "relativeDeleted");
    query(&mut server, &alias, "aliasDeleted");
    fs::write(&target, input("Child.vue.txt")).unwrap();
    query(&mut server, &fallback, "fallbackSaved");
    open(&mut server, &preferred_uri, &input("Preferred.vue.txt"));
    assert!(!preferred.exists());
    query(&mut server, &fallback, "fallbackOpenPreferred");
    close(&mut server, &preferred_uri);
    query(&mut server, &fallback, "fallbackClosedPreferred");
    fs::remove_file(&target).unwrap();
    query(&mut server, &fallback, "fallbackDeleted");
    assert!(!target.exists() && !preferred.exists());
    server.send(json!({"jsonrpc":"2.0","id":999,"method":"shutdown"}));
    assert_eq!(
        server.recv_response(999),
        json!({"jsonrpc":"2.0","id":999,"result":null})
    );
    server.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(server.wait_for_exit().success());
    if let Some(trace) = server.trace_dir() {
        fs::write(
            trace.join("open-buffer-alias-definitions.json"),
            serde_json::to_vec_pretty(&json!({
                "newline":newline,"observations":observations,
                "terminal":server.terminal_evidence(),
                "scope":"structural definition; no external Corsa required"
            }))
            .unwrap(),
        )
        .unwrap();
    }
    for observation in observations {
        assert_eq!(
            observation["actual"], observation["expected"],
            "whole {} response, newline={newline:?}",
            observation["phase"]
        );
    }
}

fn open(server: &mut LspProcess, uri: &Url, text: &str) {
    server.send(
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
            "textDocument":{"uri":uri,"languageId":"vue","version":1,"text":text}
        }}),
    );
    let publication = server.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri.as_str()
    });
    assert_eq!(
        publication,
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":[],"version":1}})
    );
}

fn close(server: &mut LspProcess, uri: &Url) {
    server.send(json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}}));
    let publication = server.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri.as_str()
    });
    assert_eq!(
        publication,
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":[]}})
    );
}
