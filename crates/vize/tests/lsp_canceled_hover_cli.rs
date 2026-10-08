#![cfg(unix)]
#![expect(clippy::disallowed_types, reason = "whole JSON-RPC process fixture")]
#![expect(
    clippy::disallowed_macros,
    reason = "authored complete Markdown oracle"
)]
#![expect(clippy::disallowed_methods, reason = "test fixture source strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "lsp_canceled_hover_cli/project.rs"]
mod project;

use lsp_process::{LspProcess, file_uri};
use project::{ORIGINAL, Project, SYNTAX, await_file};
use serde_json::{Value, json};

const DOCUMENTATION: &str = "**Primary** invoice slot shown in the summary.";

fn send(lsp: &mut LspProcess, id: i64, method: &str, params: Value) {
    let message = if params.is_null() {
        json!({"jsonrpc":"2.0","id":id,"method":method})
    } else {
        json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
    };
    lsp.send(message);
}

fn position(source: &str, offset: usize) -> Value {
    let prefix = &source[..offset];
    let start = prefix.rfind('\n').map_or(0, |at| at + 1);
    json!({"line":prefix.bytes().filter(|&byte|byte == b'\n').count(),
        "character":prefix[start..].encode_utf16().count()})
}

fn member_start(source: &str) -> usize {
    source.rfind("names.current").unwrap() + "names.".len()
}

fn hover_params(uri: &str, source: &str) -> Value {
    json!({"textDocument":{"uri":uri},"position":position(source, member_start(source)+1)})
}

fn markdown(documentation: &str) -> String {
    format!("```typescript\n(property) current: \"summary\"\n```\n{documentation}")
}

fn expected(source: &str, documentation: &str) -> Value {
    let start = member_start(source);
    json!({"contents":{"kind":"markdown","value":markdown(documentation)},
        "range":{"start":position(source,start),"end":position(source,start+"current".len())}})
}

fn open(lsp: &mut LspProcess, uri: &str, text: &str, version: i64) {
    lsp.send(
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
            "textDocument":{"uri":uri,"languageId":"vue","version":version,"text":text}
        }}),
    );
    let publication = lsp.recv_matching(|reply| {
        reply["method"] == "textDocument/publishDiagnostics"
            && reply["params"]["uri"] == uri
            && reply["params"]["version"] == version
    });
    assert_eq!(
        publication,
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
        "params":{"uri":uri,"version":version,"diagnostics":[]}})
    );
}

fn initialize(project: &Project) -> (LspProcess, String, String) {
    let root = project.root();
    let uri = file_uri(&root.join("src/SlotAuthoring.vue")).to_string();
    let syntax_uri = file_uri(&root.join("src/Syntax.vue")).to_string();
    let mut lsp = LspProcess::spawn(root);
    send(
        &mut lsp,
        1,
        "initialize",
        json!({"processId":null,"rootUri":file_uri(root),
        "capabilities":{},"initializationOptions":{"hover":true,"typecheck":true,"lint":false,"foldingRanges":true}}),
    );
    assert!(lsp.recv_response(1)["result"].is_object());
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
    (lsp, uri, syntax_uri)
}

#[test]
fn abandoned_real_native_hover_preserves_original_markdown_and_current_revision() {
    for newline in ["\n", "\r\n"] {
        for changed in [false, true] {
            let source = ORIGINAL.replace('\n', newline);
            let project = Project::new(&source);
            let (mut lsp, uri, syntax_uri) = initialize(&project);
            if let Some(capture) = lsp.trace_dir() {
                std::fs::write(capture.join("SlotAuthoring.vue"), &source).unwrap();
                std::fs::copy(
                    project.root().join("tsconfig.json"),
                    capture.join("tsconfig.json"),
                )
                .unwrap();
                std::fs::copy(
                    project.gate().join("gate.json"),
                    capture.join("native-gate-config.json"),
                )
                .unwrap();
            }
            open(&mut lsp, &syntax_uri, SYNTAX, 1);
            open(&mut lsp, &uri, &source, 1);
            send(
                &mut lsp,
                2,
                "textDocument/hover",
                hover_params(&uri, &source),
            );
            assert_eq!(
                lsp.recv_response(2),
                json!({"jsonrpc":"2.0","id":2,
                "result":expected(&source,DOCUMENTATION)})
            );

            std::fs::write(project.gate().join("arm"), "").unwrap();
            send(
                &mut lsp,
                3,
                "textDocument/hover",
                hover_params(&uri, &source),
            );
            await_file(&project.gate().join("entered"));
            let native_request: Value = serde_json::from_slice(
                &std::fs::read(project.gate().join("held-request.json")).unwrap(),
            )
            .unwrap();
            let native_response: Value = serde_json::from_slice(
                &std::fs::read(project.gate().join("held-response.json")).unwrap(),
            )
            .unwrap();
            if let Some(capture) = lsp.trace_dir() {
                std::fs::write(
                    capture.join("held-native-hover.json"),
                    serde_json::to_vec_pretty(
                        &json!({"request":native_request,"response":native_response}),
                    )
                    .unwrap(),
                )
                .unwrap();
                for name in ["held-request.bin", "held-response.bin"] {
                    std::fs::copy(project.gate().join(name), capture.join(name)).unwrap();
                }
            }
            // The actual native process has already answered. Only these bytes
            // are held; this fixture supplies no fabricated backend values.
            assert_eq!(native_request["method"], "textDocument/hover");
            let line = native_request["params"]["position"]["line"].clone();
            let character = native_request["params"]["position"]["character"]
                .as_u64()
                .unwrap();
            assert_eq!(
                native_response,
                json!({"jsonrpc":"2.0","id":native_request["id"],
                "result":{"contents":{"kind":"markdown","value":markdown(DOCUMENTATION)},
                    "range":{"start":{"line":line,"character":character-1},
                        "end":{"line":line,"character":character+6}}}})
            );
            lsp.send(json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":3}}));
            assert_eq!(
                lsp.recv_response(3),
                json!({"jsonrpc":"2.0","id":3,
                "error":{"code":-32800,"message":"Canceled"}})
            );
            send(
                &mut lsp,
                4,
                "textDocument/hover",
                hover_params(&uri, &source),
            );
            // A second waiter remains cancellable behind the abandoned IPC.
            send(
                &mut lsp,
                5,
                "textDocument/hover",
                hover_params(&uri, &source),
            );
            lsp.send(json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":5}}));
            assert_eq!(
                lsp.recv_response(5),
                json!({"jsonrpc":"2.0","id":5,
                "error":{"code":-32800,"message":"Canceled"}})
            );

            let current = if changed {
                source
                    .replace(
                        DOCUMENTATION,
                        "**Updated** invoice slot shown in the summary.",
                    )
                    .replace(
                        "<template #[names.current]",
                        "<!-- 😀 --><template #[names.current]",
                    )
            } else {
                source.clone()
            };
            if changed {
                lsp.send(
                    json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
                        "textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":current}]
                    }}),
                );
            }
            // A whole syntax response must arrive before the held typed answer.
            send(
                &mut lsp,
                6,
                "textDocument/foldingRange",
                json!({"textDocument":{"uri":syntax_uri}}),
            );
            let first = lsp.recv_matching(|reply| reply["id"] == 4 || reply["id"] == 6);
            assert_eq!(
                first,
                json!({"jsonrpc":"2.0","id":6,"result":[{
                    "startLine":0,"endLine":1,"kind":"region","collapsedText":"template"
                }]}),
            );
            std::fs::write(project.gate().join("release"), "").unwrap();
            let answer = lsp.recv_response(4);
            let expected_answer = if changed {
                json!({"jsonrpc":"2.0","id":4,"error":{"code":-32801,"message":"Content modified"}})
            } else {
                json!({"jsonrpc":"2.0","id":4,"result":expected(&source,DOCUMENTATION)})
            };
            assert_eq!(answer, expected_answer);
            let documentation = if changed {
                "**Updated** invoice slot shown in the summary."
            } else {
                DOCUMENTATION
            };
            send(
                &mut lsp,
                7,
                "textDocument/hover",
                hover_params(&uri, &current),
            );
            assert_eq!(
                lsp.recv_response(7),
                json!({"jsonrpc":"2.0","id":7,
                "result":expected(&current,documentation)})
            );
            assert_eq!(
                std::fs::read_to_string(project.root().join("src/SlotAuthoring.vue")).unwrap(),
                source
            );
            if changed {
                let current_publication = json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
                    "params":{"uri":uri,"version":2,"diagnostics":[]}});
                if !lsp
                    .published_diagnostics()
                    .iter()
                    .any(|reply| reply == &current_publication)
                {
                    let publication = lsp.recv_matching(|reply| {
                        reply["method"] == "textDocument/publishDiagnostics"
                            && reply["params"]["uri"] == uri
                            && reply["params"]["version"] == 2
                    });
                    assert_eq!(publication, current_publication);
                }
            }
            let mut declared_publications = vec![
                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
                    "params":{"uri":syntax_uri,"version":1,"diagnostics":[]}}),
                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
                    "params":{"uri":uri,"version":1,"diagnostics":[]}}),
            ];
            if changed {
                declared_publications.push(
                    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
                    "params":{"uri":uri,"version":2,"diagnostics":[]}}),
                );
            }
            send(&mut lsp, 8, "shutdown", Value::Null);
            assert_eq!(
                lsp.recv_response(8),
                json!({"jsonrpc":"2.0","id":8,"result":null})
            );
            lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
            assert!(
                lsp.wait_for_exit().success(),
                "exit must finish with stdin still open"
            );
            // Exit joins both readers: compare every retained whole publication,
            // including messages consumed while matching semantic responses.
            for publication in lsp.published_diagnostics() {
                let expected = declared_publications
                    .iter()
                    .find(|row| {
                        row["params"]["uri"] == publication["params"]["uri"]
                            && row["params"]["version"] == publication["params"]["version"]
                    })
                    .expect("publication must belong to a declared document/revision");
                assert_eq!(&publication, expected);
            }
        }
    }
}
