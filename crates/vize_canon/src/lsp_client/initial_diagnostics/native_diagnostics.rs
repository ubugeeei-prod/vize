//! Compiler-kind ABI adapter, retaining one immutable SDK lease.
//! Authority: microsoft/typescript-go@2bd066d87f5bafd315be9f40889d0a60b9e58e0b
//! internal/{api/{proto,session},ls/diagnostics,core/{compileroptions,tristate}}.go
//! (MIT, Copyright Microsoft). Only the per-file PULL union is implemented:
//! no Bind/global/options/config scope, aggregate endpoint or capability is lent.
//! The captured lease survives every RPC drain; each next RPC checks retirement.
//! The existing SINGLE worker deadline covers this read AND fallback. Text comes
//! from that same snapshot, never current disk, overlays or another snapshot.

use std::path::Path;

use corsa::{
    CorsaError,
    api::{ApiClient, ManagedSnapshot, ProjectResponse},
    runtime::block_on,
};
use lsp_types::Diagnostic;
use serde_json::{Value, json};
use vize_l0::{FxHashMap, String, cstr};

use super::diagnostic_response::{NativeDiagnostic, SnapshotText, missing_method};
use crate::{
    corsa_bridge::native_operation,
    file_uri::{file_uri_to_path, path_to_typescript_name},
    lsp_client::editor_lsp::snapshot_source::{SourceTextRefusal, decode_snapshot_text},
};

enum Failure {
    Unsupported,
    Failed(String),
}

/// No mutable client access, new snapshot/project selection or state commit.
pub(in crate::lsp_client) fn read(
    client: &ApiClient,
    snapshot: &ManagedSnapshot,
    project: &ProjectResponse,
    document_uri: &str,
) -> Result<Option<Vec<Diagnostic>>, String> {
    let result = (|| {
        let path = file_uri_to_path(document_uri)
            .filter(|path| path.is_absolute())
            .ok_or(Failure::Unsupported)?;
        let file = path_to_typescript_name(&path);
        let params = json!({"snapshot":snapshot.handle,"project":project.id,"file":file});
        let mut rows = Vec::new();
        for method in required_kinds(&project.compiler_options)? {
            checkpoint()?;
            let response = block_on(client.raw_json_request(method, params.clone()))
                .map_err(|error| classify(method, error))?;
            super::trace_kind(&params, method, &response);
            // Native NewDiagnosticResponses(nil) serializes a successful null.
            // It is empty for THIS kind only; every remaining kind must succeed.
            if !response.is_null() {
                rows.extend(
                    serde_json::from_value::<Vec<NativeDiagnostic>>(response).map_err(|error| {
                        Failure::Failed(cstr!("Malformed {method} response: {error}"))
                    })?,
                );
            }
        }
        if rows.iter().any(|row| !row.belongs_to(&file)) {
            return Err(Failure::Failed(cstr!(
                "Native file diagnostic belongs to another source"
            )));
        }
        let mut texts = FxHashMap::<String, SnapshotText>::default();
        for name in rows.iter().flat_map(NativeDiagnostic::source_names) {
            if texts.contains_key(name) {
                continue;
            }
            if !Path::new(name).is_absolute() || name.contains('\0') {
                return Err(Failure::Failed(cstr!("Invalid native source identity")));
            }
            checkpoint()?;
            let encoded =
                block_on(client.get_source_file(snapshot.handle.clone(), project.id.clone(), name))
                    .map_err(|error| classify("getSourceFile", error))?
                    .ok_or(Failure::Unsupported)?;
            let text = match decode_snapshot_text(&encoded, name) {
                Ok(text) => text,
                Err(SourceTextRefusal::Version) => return Err(Failure::Unsupported),
                Err(reason) => {
                    return Err(Failure::Failed(cstr!(
                        "Malformed native snapshot source {name}: {reason:?}"
                    )));
                }
            };
            super::trace_source(&params, name, text);
            texts.insert(
                name.into(),
                SnapshotText::new(text).map_err(Failure::Failed)?,
            );
        }
        checkpoint()?;
        rows.into_iter()
            .map(|row| {
                row.into_lsp(&file, |name| {
                    texts
                        .get(name)
                        .ok_or_else(|| cstr!("Missing native snapshot source {name}"))
                })
                .map_err(Failure::Failed)
            })
            .collect::<Result<Vec<_>, _>>()
    })();
    match result {
        Ok(rows) => Ok(Some(rows)),
        Err(Failure::Unsupported) => Ok(None),
        Err(Failure::Failed(error)) => Err(error),
    }
}

fn checkpoint() -> Result<(), Failure> {
    native_operation::checkpoint().map_err(Failure::Failed)
}

fn required_kinds(options: &Value) -> Result<Vec<&'static str>, Failure> {
    let options = options.as_object().ok_or_else(|| {
        Failure::Failed(cstr!("Native project compiler options are not an object"))
    })?;
    let mut declarations = false;
    for key in ["declaration", "composite"] {
        match options.get(key) {
            None | Some(Value::Null) | Some(Value::Bool(false)) => {}
            Some(Value::Bool(true)) => declarations = true,
            Some(_) => {
                return Err(Failure::Failed(cstr!(
                    "Invalid native {key} compiler option"
                )));
            }
        }
    }
    let mut methods = vec![
        "getSyntacticDiagnostics",
        "getSemanticDiagnostics",
        "getSuggestionDiagnostics",
    ];
    if declarations {
        methods.push("getDeclarationDiagnostics");
    }
    Ok(methods)
}

fn classify(method: &str, error: CorsaError) -> Failure {
    if missing_method(method, &error) {
        Failure::Unsupported
    } else {
        Failure::Failed(cstr!("Failed native {method} read: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::String;
    use super::{
        ApiClient, CorsaError, Diagnostic, Value, block_on, cstr, json, read, required_kinds,
    };
    const URI: &str = "file:///workspace/a%20%23.ts";
    const KINDS: [&str; 4] = [
        "getSyntacticDiagnostics",
        "getSemanticDiagnostics",
        "getSuggestionDiagnostics",
        "getDeclarationDiagnostics",
    ];
    fn diagnostic_methods(rows: &[Value]) -> Vec<&str> {
        rows.iter()
            .filter_map(|row| row["method"].as_str())
            .filter(|method| {
                !matches!(
                    *method,
                    "initialize" | "updateSnapshot" | "release" | "ping" | "test/finish"
                )
            })
            .collect()
    }
    #[test]
    fn captured_declaration_options_select_exact_ls_union_not_extra_scopes() {
        let three = KINDS.iter().take(3).copied().collect::<Vec<_>>();
        let mut four = three.clone();
        four.push("getDeclarationDiagnostics");
        for options in [json!({}), json!({"declaration":false,"composite":null})] {
            assert_eq!(required_kinds(&options).ok(), Some(three.clone()));
        }
        for options in [
            json!({"declaration":true}),
            json!({"composite":true,"declaration":false}),
        ] {
            assert_eq!(required_kinds(&options).ok(), Some(four.clone()));
        }
        for options in [
            Value::Null,
            json!({"declaration":2}),
            json!({"composite":"true"}),
        ] {
            assert!(required_kinds(&options).is_err());
        }
    }

    #[cfg(unix)]
    struct Peer {
        client: ApiClient,
        _directory: tempfile::TempDir,
        entered: std::sync::mpsc::Receiver<()>,
        thread: std::thread::JoinHandle<Vec<serde_json::Value>>,
    }
    #[cfg(unix)]
    impl Peer {
        fn start(
            options: Value,
            replies: Vec<(&'static str, Value)>,
            hold: Option<&'static str>,
        ) -> Self {
            use corsa::jsonrpc::{RawMessage, read_frame, write_frame};
            use std::io::{BufReader, BufWriter};
            use std::os::unix::net::UnixListener;
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("lower.sock");
            let listener = UnixListener::bind(&path).unwrap();
            let (entered, received) = std::sync::mpsc::sync_channel(1);
            let thread = std::thread::spawn(move || {
                let (socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(std::time::Duration::from_secs(10)))
                    .unwrap();
                socket
                    .set_write_timeout(Some(std::time::Duration::from_secs(10)))
                    .unwrap();
                let mut reader = BufReader::new(socket.try_clone().unwrap());
                let mut writer = BufWriter::new(socket);
                let mut replies = std::collections::VecDeque::from(replies);
                let mut observed = Vec::new();
                let mut pending = None;
                let mut generation = 0;
                let mut released = Vec::new();
                loop {
                    let body = match read_frame(&mut reader) {
                        Ok(body) => body,
                        Err(CorsaError::Closed(_)) => break,
                        Err(error) => panic!("lower peer: {error}"),
                    };
                    let request: RawMessage = serde_json::from_slice(&body).unwrap();
                    let id = request.id.clone().unwrap();
                    let method = request.method.as_deref().unwrap();
                    let params = request.params.clone().unwrap_or(Value::Null);
                    observed.push(serde_json::to_value(&request).unwrap());
                    if matches!(
                        method,
                        "getSyntacticDiagnostics"
                            | "getSemanticDiagnostics"
                            | "getSuggestionDiagnostics"
                            | "getDeclarationDiagnostics"
                            | "getSourceFile"
                    ) {
                        assert_eq!(
                            params,
                            json!({"snapshot":"n0000000000000001",
                            "project":"p./workspace/tsconfig.json","file":"/workspace/a #.ts"})
                        );
                    }
                    let result = match method {
                        "initialize" => {
                            json!({"useCaseSensitiveFileNames":true,"currentDirectory":"/workspace"})
                        }
                        "updateSnapshot" => {
                            generation += 1;
                            json!({"snapshot":if generation==1 {"n0000000000000001"} else {"n0000000000000002"},
                                "projects":[{"id":"p./workspace/tsconfig.json","configFileName":"/workspace/tsconfig.json",
                                "compilerOptions":options,"rootFiles":["/workspace/a #.ts"]}],
                                "changes":{"changedProjects":{},"removedProjects":[]}})
                        }
                        "release" | "test/barrier" => Value::Null,
                        "ping" => json!("foreground live"),
                        "test/finish" => {
                            let previous = pending.take().unwrap();
                            write_frame(
                                &mut writer,
                                &serde_json::to_vec(&RawMessage::response(previous, Value::Null))
                                    .unwrap(),
                            )
                            .unwrap();
                            Value::Null
                        }
                        method if hold == Some(method) => {
                            assert!(pending.replace(id).is_none());
                            entered.send(()).unwrap();
                            continue;
                        }
                        _ => {
                            let (expected, response) =
                                replies.pop_front().expect("unexpected lower RPC");
                            assert_eq!(method, expected);
                            if let Some(error) = response.get("error") {
                                let packet = json!({"jsonrpc":"2.0","id":id,"error":error});
                                write_frame(&mut writer, &serde_json::to_vec(&packet).unwrap())
                                    .unwrap();
                                continue;
                            }
                            response
                        }
                    };
                    write_frame(
                        &mut writer,
                        &serde_json::to_vec(&RawMessage::response(id, result)).unwrap(),
                    )
                    .unwrap();
                    if method == "release" {
                        assert_eq!(
                            params,
                            json!({"handle":params["handle"], "snapshot":params["handle"]})
                        );
                        released.push(params["handle"].clone());
                        if released.len() == if hold.is_some() { 2 } else { 1 } {
                            assert!(pending.is_none());
                            break;
                        }
                    }
                }
                assert!(replies.is_empty());
                released.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
                let expected = (1..=if hold.is_some() { 2 } else { 1 })
                    .map(|revision| json!(cstr!("n{revision:016x}")))
                    .collect::<Vec<_>>();
                assert_eq!(released, expected);
                observed
            });
            let client = block_on(ApiClient::connect_pipe(&path)).unwrap();
            Self {
                client,
                _directory: directory,
                entered: received,
                thread,
            }
        }

        fn finish(self) -> Vec<Value> {
            block_on(self.client.close()).unwrap();
            self.thread.join().unwrap()
        }
    }
    #[cfg(unix)]
    fn execute(
        options: Value,
        replies: Vec<(&'static str, Value)>,
    ) -> (Result<Option<Vec<Diagnostic>>, String>, Vec<Value>) {
        let peer = Peer::start(options, replies, None);
        let snapshot = block_on(peer.client.update_snapshot(Default::default())).unwrap();
        let result = read(
            &peer.client,
            &snapshot,
            snapshot.projects.first().unwrap(),
            URI,
        );
        block_on(snapshot.release()).unwrap();
        (result, peer.finish())
    }
    #[cfg(unix)]
    #[test]
    fn framed_null_empty_and_declaration_union_keep_one_snapshot_and_exact_order() {
        for declarations in [false, true] {
            let mut replies = vec![
                ("getSyntacticDiagnostics", Value::Null),
                ("getSemanticDiagnostics", json!([])),
                ("getSuggestionDiagnostics", Value::Null),
            ];
            if declarations {
                replies.push(("getDeclarationDiagnostics", json!([])));
            }
            let expected = replies
                .iter()
                .map(|(method, _)| *method)
                .collect::<Vec<_>>();
            let (result, observed) = execute(json!({"declaration":declarations}), replies);
            assert_eq!(result.unwrap(), Some(vec![]));
            assert_eq!(diagnostic_methods(&observed), expected);
        }
    }
    #[cfg(unix)]
    #[test]
    fn framed_partial_kind_missing_failure_and_malformed_never_become_empty_success() {
        let kinds = KINDS;
        for (index, kind) in kinds.iter().enumerate() {
            for response in [
                json!({"error":{"code":-32601,"message":"method not found"}}),
                json!({"error":{"code":-32603,"message":cstr!("invalid request: unknown API method \"{kind}\"")}}),
                json!({"error":{"code":-32602,"message":cstr!("invalid request: unknown API method \"{kind}\"")}}),
                json!({"error":{"code":-32000,"message":cstr!("invalid request: unknown API method \"{kind}\"")}}),
                json!({"error":{"code":-32000,"message":"snapshot retired"}}),
                json!({"not":"a diagnostic list"}),
            ] {
                let missing =
                    response["error"]["code"] == -32601 || response["error"]["code"] == -32603;
                let mut replies=kinds.iter().take(index).map(|method|(*method,json!([
                    {"fileName":"/workspace/a #.ts","pos":0,"end":5,"code":2322,"category":1,"text":"must not publish partial"}
                ]))).collect::<Vec<_>>();
                replies.push((*kind, response));
                let (result, observed) = execute(json!({"composite":true}), replies);
                if missing {
                    assert_eq!(result.unwrap(), None);
                } else {
                    assert!(result.is_err());
                }
                let methods = diagnostic_methods(&observed);
                assert_eq!(
                    methods,
                    kinds.iter().take(index + 1).copied().collect::<Vec<_>>()
                );
                assert!(!observed.iter().any(|row| row["method"] == "getSourceFile"));
            }
        }
    }
    #[cfg(unix)]
    #[test]
    fn framed_complete_result_uses_original_snapshot_encoded_text_not_live_disk() {
        let data = ENCODED_TEXT;
        let row = |code, text| {
            json!({"fileName":"/workspace/a #.ts","pos":0,"end":5,
            "code":code,"category":1,"text":text})
        };
        let (result, observed) = execute(
            json!({"declaration":true}),
            vec![
                ("getSyntacticDiagnostics", json!([row(1005, "syntactic")])),
                ("getSemanticDiagnostics", json!([row(2322, "semantic")])),
                ("getSuggestionDiagnostics", json!([row(6133, "suggestion")])),
                (
                    "getDeclarationDiagnostics",
                    json!([row(4023, "declaration")]),
                ),
                ("getSourceFile", json!({"data":data})),
            ],
        );
        let actual = serde_json::to_value(result.unwrap().unwrap()).unwrap();
        let expected=vec![(1005,1,"syntactic"),(2322,1,"semantic"),(6133,2,"suggestion"),(4023,1,"declaration")]
            .into_iter().map(|(code,severity,text)|json!({"range":{"start":{"line":0,"character":0},
                "end":{"line":0,"character":5}},"code":code,"severity":severity,"source":"ts","message":text})).collect::<Vec<_>>();
        assert_eq!(actual, json!(expected));
        assert_eq!(
            observed
                .iter()
                .filter(|row| row["method"] == "getSourceFile")
                .count(),
            1
        );
    }
    #[cfg(unix)]
    #[test]
    fn cancelled_lower_rpc_drains_actual_arc_lease_without_starting_next_kind() {
        use crate::corsa_bridge::worker::BoundedWorker;
        use futures::FutureExt;
        use std::{sync::Arc, time::Duration};
        let kinds = KINDS;
        for (index, kind) in kinds.iter().enumerate() {
            let prior = kinds.iter().take(index).map(|method| (*method,json!([{
            "fileName":"/workspace/a #.ts","pos":0,"end":5,"code":2322,"category":1,"text":"partial must not publish"
        }]))).collect();
            let peer = Peer::start(json!({"declaration":true}), prior, Some(*kind));
            let snapshot =
                Arc::new(block_on(peer.client.update_snapshot(Default::default())).unwrap());
            let weak = Arc::downgrade(&snapshot);
            let worker = BoundedWorker::new("lower diagnostic lease", ());
            let client = peer.client.clone();
            let owned = Arc::clone(&snapshot);
            let deadline = Duration::from_secs(10);
            let mut pending = Box::pin(worker.submit_ready_async(deadline, move |_| {
                read(&client, &owned, owned.projects.first().unwrap(), URI)
            }));
            assert!(pending.as_mut().now_or_never().is_none());
            peer.entered.recv_timeout(Duration::from_secs(10)).unwrap();
            drop(pending);
            drop(snapshot);
            assert!(worker.is_draining());
            assert!(weak.upgrade().is_some());
            let newer = block_on(peer.client.update_snapshot(Default::default())).unwrap();
            assert_eq!(
                block_on(peer.client.raw_json_request("ping", Value::Null)).unwrap(),
                json!("foreground live")
            );
            block_on(peer.client.raw_json_request("test/finish", Value::Null)).unwrap();
            block_on(worker.submit_ready_async(Duration::from_secs(10), |_| ())).unwrap();
            assert!(weak.upgrade().is_none());
            assert!(!worker.is_draining());
            block_on(newer.release()).unwrap();
            let observed = peer.finish();
            let methods = diagnostic_methods(&observed);
            assert_eq!(
                methods,
                kinds.iter().take(index + 1).copied().collect::<Vec<_>>()
            );
            assert!(!observed.iter().any(|row| row["method"] == "getSourceFile"));
            let releases = observed
                .iter()
                .filter(|row| row["method"] == "release")
                .count();
            assert_eq!(releases, 2);
        }
    }
    #[cfg(unix)]
    const ENCODED_TEXT: &str = "AAAABQAAAAAAAAAAAAAAAAAAAAAAAAAALAAAAEQAAACGAAAAtgAAALYAAAAAAAAAIAAAACAAAAAxAAAAMQAAAEIAAABjb25zdCBlbW9qaSA9ICfwn5iAJzsNCi8vIGNhZsOpDS93b3Jrc3BhY2UvYSAjLnRzL3dvcmtzcGFjZS9hICMudHMAAAAAAgAAAAQAAAAAAAAAAwAAAP///////////////////////////////wAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMwEAAAAAAAAdAAAAAAAAAAAAAAAAAACAAAAAAA==";
}
