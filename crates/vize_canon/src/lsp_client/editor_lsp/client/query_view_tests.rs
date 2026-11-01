//! Real framed peer controls query views; no native semantic qualification.

use super::{CorsaProjectClient, FxHashMap};
use crate::file_uri::path_to_file_uri;
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt};
use vize_l0::String;

#[test]
fn logical_query_subsets_preserve_retained_buffers_but_explicit_close_removes_them() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let peer = root.join("query-peer.ts");
    let transcript = root.join("whole-packets.jsonl");
    fs::write(
        &peer,
        include_str!("../native_shutdown/peer.ts.txt")
            .replace("LOG_PATH", &serde_json::to_string(&transcript).unwrap())
            .replace("REPLY_MODE", "\"ok\""),
    )
    .unwrap();
    fs::set_permissions(&peer, fs::Permissions::from_mode(0o755)).unwrap();
    let mut client = CorsaProjectClient::batch_editor_peer_for_test(&peer, &root).unwrap();
    client.logical_workspace_overlays = true;
    let main = path_to_file_uri(&root.join("main.ts"));
    let support = path_to_file_uri(&root.join("support.ts"));
    let virtual_uri = path_to_file_uri(&root.join("Virtual.vue.ts"));
    let transient = path_to_file_uri(&root.join("closed-query.ts"));
    let retained = FxHashMap::from_iter([
        (main.clone(), String::from("export const main = 1;")),
        (support.clone(), String::from("export const support = 2;")),
        (
            virtual_uri.clone(),
            String::from("export const virtualValue = 3;"),
        ),
    ]);
    client.document_texts = retained.clone();
    let report = json!({"kind":"full","items":[]});
    assert_eq!(
        serde_json::to_value(client.diagnostics_via_editor_lsp(&main, &retained).unwrap()).unwrap(),
        report,
    );
    let mut subset = retained.clone();
    subset.remove(&support);
    for view in [&subset, &retained] {
        assert_eq!(
            serde_json::to_value(client.diagnostics_via_editor_lsp(&main, view).unwrap()).unwrap(),
            report,
        );
        assert_eq!(client.document_texts, retained);
        assert!(!client.editor_lsp_documents_dirty);
    }
    client.did_close(&support).unwrap();
    assert_eq!(client.document_texts, subset);
    let mut temporary = subset.clone();
    temporary.insert(main.clone(), "export const main = 'temporary';".into());
    temporary.insert(transient.clone(), "export const closedQuery = 4;".into());
    assert_eq!(
        serde_json::to_value(
            client
                .diagnostics_via_editor_lsp(&main, &temporary)
                .unwrap()
        )
        .unwrap(),
        report,
    );
    assert_eq!(client.document_texts, subset);
    assert!(client.editor_lsp_documents_dirty);
    assert_eq!(
        serde_json::to_value(client.diagnostics_via_editor_lsp(&main, &subset).unwrap()).unwrap(),
        report,
    );
    assert!(!client.editor_lsp_documents_dirty);
    assert_eq!(client.document_texts, subset);
    client.shutdown().unwrap();
    let packets = fs::read_to_string(&transcript)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    let document_events = |uri: &str| {
        packets
            .iter()
            .filter(|packet| packet["direction"] == "request")
            .map(|packet| &packet["wholePacket"])
            .filter(|packet| packet["params"]["textDocument"]["uri"] == uri)
            .filter_map(|packet| {
                let method = packet["method"].as_str()?;
                matches!(
                    method,
                    "textDocument/didOpen" | "textDocument/didChange" | "textDocument/didClose"
                )
                .then(|| packet.clone())
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        document_events(&support),
        vec![
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":support,"languageId":"typescript","version":1,"text":"export const support = 2;"}}}),
            json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":support}}}),
        ],
        "{packets:#?}",
    );
    assert_eq!(
        document_events(&transient),
        vec![
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":transient,"languageId":"typescript","version":1,"text":"export const closedQuery = 4;"}}}),
            json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":transient}}}),
        ],
        "{packets:#?}",
    );
    assert_eq!(
        document_events(&main),
        vec![
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":main,"languageId":"typescript","version":1,"text":"export const main = 1;"}}}),
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":main,"version":2},"contentChanges":[{"text":"export const main = 'temporary';"}]}}),
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":main,"version":3},"contentChanges":[{"text":"export const main = 1;"}]}}),
        ],
        "{packets:#?}",
    );
    assert_eq!(packets.last(), Some(&json!({"event":"stdin-eof"})));
}
