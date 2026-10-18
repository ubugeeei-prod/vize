use super::{MaterializedDelta, MaterializedSnapshot, VirtualProject};
use crate::batch::source_policy::SourceFilePolicy;
use std::path::PathBuf;
use vize_carton::FxHashMap;

#[test]
fn unsupported_explicit_editor_selection_uses_only_the_exact_internal_fallback_reason() {
    use crate::batch::error::CorsaError;
    let reason = crate::lsp_client::EXPLICIT_CONFIG_ATTACHMENT_UNSUPPORTED;
    assert!(super::explicit_config_attachment_unsupported(
        &CorsaError::CorsaExecution {
            exit_code: -1,
            message: reason.into(),
        }
    ));
    for message in [
        "invalid generated configuration",
        "Cannot open explicit diagnosing project",
    ] {
        assert!(!super::explicit_config_attachment_unsupported(
            &CorsaError::CorsaExecution {
                exit_code: -1,
                message: message.into(),
            }
        ));
    }
}

#[test]
fn snapshot_diff_classifies_created_changed_and_deleted_files() {
    let previous = snapshot(&[("/virtual/changed.ts", 1), ("/virtual/deleted.ts", 2)]);
    let current = snapshot(&[("/virtual/changed.ts", 3), ("/virtual/created.ts", 4)]);

    assert_eq!(
        current.diff(&previous),
        MaterializedDelta {
            changed: vec![PathBuf::from("/virtual/changed.ts")],
            created: vec![PathBuf::from("/virtual/created.ts")],
            deleted: vec![PathBuf::from("/virtual/deleted.ts")],
        }
    );
}

#[cfg(unix)]
#[test]
fn snapshot_keeps_symlinked_typescript_files_as_diagnostic_inputs() {
    use crate::file_uri::path_to_file_uri;
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temporary project should be created");
    let virtual_root = temp.path().join("canon");
    let target = temp.path().join("external.ts");
    let linked = virtual_root.join("linked.ts");
    std::fs::create_dir_all(&virtual_root).expect("virtual root should be created");
    std::fs::write(&target, "export const value = 1\n").expect("target should be written");
    symlink(&target, &linked).expect("source symlink should be created");

    let before = MaterializedSnapshot::capture(&virtual_root, SourceFilePolicy::default())
        .expect("materialized snapshot should be captured");
    std::fs::write(&target, "export const value = 2\n").expect("target should be updated");
    let after = MaterializedSnapshot::capture(&virtual_root, SourceFilePolicy::default())
        .expect("updated materialized snapshot should be captured");

    assert!(after.revisions.contains_key(&linked));
    assert_eq!(after.uris, vec![path_to_file_uri(&linked)]);
    assert_eq!(after.diff(&before).changed, vec![linked]);
}

#[test]
fn snapshot_tracks_in_place_diagnostic_source_changes() {
    use crate::file_uri::path_to_file_uri;

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("aliased.ts");
    std::fs::write(&source, "export const value = 1\n").unwrap();
    let mut project = VirtualProject::new(temp.path()).unwrap();
    std::fs::create_dir_all(project.virtual_root()).unwrap();
    project.set_diagnostic_paths([source.as_path()]);
    let mut before =
        MaterializedSnapshot::capture(project.virtual_root(), SourceFilePolicy::default()).unwrap();
    before.extend_diagnostic_paths(&project).unwrap();

    std::fs::write(&source, "export const value = 2\n").unwrap();
    let mut after =
        MaterializedSnapshot::capture(project.virtual_root(), SourceFilePolicy::default()).unwrap();
    after.extend_diagnostic_paths(&project).unwrap();

    let source = source.canonicalize().unwrap();
    assert_eq!(after.uris, vec![path_to_file_uri(&source)]);
    assert_eq!(after.diff(&before).changed, vec![source]);
}

#[cfg(unix)]
#[test]
fn incremental_snapshot_hashes_directory_symlink_retargets_without_reading_the_directory() {
    use crate::batch::virtual_project::MaterializedFileDelta;
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    let project = VirtualProject::new(temp.path()).unwrap();
    let linked = project.virtual_root().join("node_modules/@scope/ui");
    std::fs::create_dir_all(linked.parent().unwrap()).unwrap();
    symlink(&first, &linked).unwrap();

    let mut snapshot = MaterializedSnapshot::default();
    snapshot
        .apply_delta(
            &project,
            &MaterializedFileDelta {
                created: vec![linked.clone()],
                ..Default::default()
            },
        )
        .unwrap();
    let first_revision = snapshot.revisions[&linked];

    std::fs::remove_file(&linked).unwrap();
    symlink(&second, &linked).unwrap();
    snapshot
        .apply_delta(
            &project,
            &MaterializedFileDelta {
                changed: vec![linked.clone()],
                ..Default::default()
            },
        )
        .unwrap();

    assert_ne!(snapshot.revisions[&linked], first_revision);
    assert!(snapshot.uris.is_empty());
}

fn snapshot(entries: &[(&str, u64)]) -> MaterializedSnapshot {
    MaterializedSnapshot {
        revisions: entries
            .iter()
            .map(|(path, revision)| (PathBuf::from(path), *revision))
            .collect::<FxHashMap<_, _>>(),
        uris: Vec::new(),
    }
}

#[cfg(unix)]
#[test]
fn batch_waits_for_owned_shutdown_and_keeps_primary_diagnostic_failure() {
    use super::CorsaExecutor;
    use crate::{batch::error::CorsaError, corsa_client::CorsaProjectClient};
    use serde_json::{Value, json};
    use std::{fs, os::unix::fs::PermissionsExt};
    use vize_l0::cstr;

    for diagnostic_mode in ["clean", "semantic", "error"] {
        for shutdown_mode in ["ok", "error"] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap();
            let script = root.join("peer.ts");
            let transcript = root.join("whole-packets.jsonl");
            let peer = include_str!("../../../lsp_client/editor_lsp/native_shutdown/peer.ts.txt")
                .replace("LOG_PATH", &serde_json::to_string(&transcript).unwrap())
                .replace(
                    "REPLY_MODE",
                    &json!({"shutdown":shutdown_mode,"diagnostics":diagnostic_mode}).to_string(),
                );
            fs::write(&script, peer).unwrap();
            fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
            let source = root.join("index.ts");
            let authored = "const value: number = 'bad';\n";
            fs::write(&source, authored).unwrap();
            let mut project = VirtualProject::new(&root).unwrap();
            fs::create_dir_all(project.virtual_root()).unwrap();
            project.set_diagnostic_paths([source.as_path()]);
            let uri = crate::file_uri::path_to_file_uri(&source);
            let client = CorsaProjectClient::batch_editor_peer_for_test(&script, &root).unwrap();
            let executor = CorsaExecutor::with_corsa_path(&root, Some(&script)).unwrap();
            let result = executor.check_with_initialized_project_session(client, &project);

            let diagnostic_error = cstr!(
                "Failed to request LSP diagnostics for {uri}: Failed to request editor LSP diagnostics for {uri}: rpc error -32603: controlled diagnostic refusal"
            );
            let close_error = cstr!(
                "Failed to gracefully close editor LSP process: rpc error -32603: controlled shutdown refusal"
            );
            if diagnostic_mode == "error" || shutdown_mode == "error" {
                let Err(CorsaError::CorsaExecution { exit_code, message }) = result else {
                    panic!("batch completion must retain the actual owner failure: {result:?}")
                };
                assert_eq!(exit_code, -1);
                assert_eq!(
                    message,
                    if diagnostic_mode == "error" {
                        diagnostic_error
                    } else {
                        close_error
                    }
                );
            } else {
                let result = result.unwrap();
                let diagnostics: Vec<_> = result
                    .diagnostics
                    .iter()
                    .map(|d| {
                        json!({
                            "file":d.file,"line":d.line,"column":d.column,"code":d.code,
                    "severity":d.severity,"message":d.message,"blockType":d.block_type.map(|block|block.block_name()),
                        })
                    })
                    .collect();
                let expected = if diagnostic_mode == "semantic" {
                    vec![json!({"file":source,"line":0,"column":0,"code":2322,
                        "severity":1,"message":"Type 'string' is not assignable to type 'number'.",
                        "blockType":null})]
                } else {
                    Vec::new()
                };
                assert_eq!(
                    (result.success, result.exit_code, diagnostics),
                    (
                        diagnostic_mode == "clean",
                        i32::from(diagnostic_mode == "semantic"),
                        expected
                    )
                );
            }

            let frames: Vec<Value> = fs::read_to_string(&transcript)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            let root_uri = crate::file_uri::path_to_file_uri(&root);
            let mut expected = vec![
                json!({"direction":"request","wholePacket":{
                    "jsonrpc":"2.0","id":1,"method":"initialize","params":{
                        "processId":std::process::id(),
                        "rootPath":root.to_str().unwrap(),
                        "rootUri":root_uri,
                        "workspaceFolders":[{"uri":root_uri,"name":root.file_name().unwrap().to_str().unwrap()}],
                        "capabilities":{
                            "workspace":{
                                "didChangeWatchedFiles":{"dynamicRegistration":true,"relativePatternSupport":true},
                                "diagnostic":{"refreshSupport":true}
                            },
                            "textDocument":{
                                "hover":{"contentFormat":["markdown","plaintext"]},
                                "completion":{"completionItem":{"documentationFormat":["markdown","plaintext"]}},
                                "signatureHelp":{"signatureInformation":{"documentationFormat":["markdown","plaintext"]}},
                                "diagnostic":{"dynamicRegistration":false,"relatedDocumentSupport":true,"relatedInformation":true}
                            }
                        },
                        "initializationOptions":{"userPreferences":{
                            "tsserver":{"automaticTypeAcquisition":{"enabled":false}},
                            "inlayHints":{"variableTypes":{"enabled":true}}
                        }}
                    }
                }}),
                json!({"direction":"response","wholePacket":{"jsonrpc":"2.0","id":1,"result":{"capabilities":{}}}}),
                json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","method":"initialized","params":{}}}),
                json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"typescript","version":1,"text":authored}}}}),
                json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","id":2,"method":"textDocument/documentSymbol","params":{"textDocument":{"uri":uri}}}}),
                json!({"direction":"response","wholePacket":{"jsonrpc":"2.0","id":2,"result":[]}}),
                json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","id":3,"method":"textDocument/diagnostic","params":{"textDocument":{"uri":uri}}}}),
            ];
            let items = if diagnostic_mode == "semantic" {
                json!([{
                    "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},
                    "severity":1,"code":2322,"message":"Type 'string' is not assignable to type 'number'."
                }])
            } else {
                json!([])
            };
            let response = if diagnostic_mode == "error" {
                json!({"jsonrpc":"2.0","id":3,"error":{"code":-32603,"message":"controlled diagnostic refusal","data":{"phase":"diagnostics"}}})
            } else {
                json!({"jsonrpc":"2.0","id":3,"result":{"kind":"full","items":items}})
            };
            expected.push(json!({"direction":"response","wholePacket":response}));
            expected.push(json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","id":4,"method":"shutdown"}}));
            let response = if shutdown_mode == "error" {
                json!({"jsonrpc":"2.0","id":4,"error":{"code":-32603,"message":"controlled shutdown refusal","data":{"phase":"shutdown"}}})
            } else {
                json!({"jsonrpc":"2.0","id":4,"result":null})
            };
            expected.push(json!({"direction":"response","wholePacket":response}));
            if shutdown_mode == "ok" {
                expected.push(
                    json!({"direction":"request","wholePacket":{"jsonrpc":"2.0","method":"exit"}}),
                );
            }
            expected.push(json!({"event":"stdin-eof"}));
            assert_eq!(frames.len(), expected.len());
            assert_eq!(frames, expected, "{diagnostic_mode}/{shutdown_mode}");
        }
    }
}
