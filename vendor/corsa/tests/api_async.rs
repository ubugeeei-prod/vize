mod support;

use std::{fs, thread, time::Duration};

use corsa::api::{
    ApiClient, ApiMode, DocumentIdentifier, OverlayChanges, OverlayUpdate, PrintNodeOptions,
    UpdateSnapshotParams,
};
use corsa::runtime::block_on;
use serde_json::json;

#[test]
fn async_api_roundtrip_core() {
    block_on(async {
        let client = ApiClient::spawn(
            support::api_config(ApiMode::AsyncJsonRpcStdio)
                .with_allow_unstable_upstream_calls(true),
        )
        .await
        .unwrap();
        let init = client.initialize().await.unwrap();
        assert_eq!(
            init.current_directory,
            support::test_cwd().display().to_string()
        );
        let config = client
            .parse_config_file("/workspace/tsconfig.json")
            .await
            .unwrap();
        assert_eq!(config.file_names, vec!["/workspace/src/index.ts"]);
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some("/workspace/tsconfig.json".into()),
                file_changes: None,
                overlay_changes: None,
            })
            .await
            .unwrap();
        assert_eq!(snapshot.projects.len(), 1);
        let project = snapshot.projects[0].id.clone();
        let default = snapshot
            .get_default_project_for_file("/workspace/src/index.ts")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(default.id.as_str(), project.as_str());
        let source = client
            .get_source_file(
                snapshot.handle.clone(),
                project.clone(),
                "/workspace/src/index.ts",
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(source.as_bytes(), b"source-file");
        let symbol = client
            .get_symbol_at_position(
                snapshot.handle.clone(),
                project.clone(),
                "/workspace/src/index.ts",
                1,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(symbol.name, "value");
        let ty = client
            .get_type_of_symbol(snapshot.handle.clone(), project.clone(), symbol.id.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(ty.id.as_str(), "t0000000000000001");
        let rendered = client
            .type_to_string(
                snapshot.handle.clone(),
                project.clone(),
                ty.id.clone(),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(rendered, "type:string");
        let printed = client
            .print_node(
                &corsa::api::EncodedPayload::new(b"hello".to_vec()),
                PrintNodeOptions::default(),
            )
            .await
            .unwrap();
        assert_eq!(printed, "print:hello");
        snapshot.release().await.unwrap();
        client.close().await.unwrap();
    });
}

#[test]
fn async_api_rejects_unstable_print_node_by_default() {
    block_on(async {
        let client = ApiClient::spawn(support::api_config(ApiMode::AsyncJsonRpcStdio))
            .await
            .unwrap();
        let error = client
            .print_node(
                &corsa::api::EncodedPayload::new(b"hello".to_vec()),
                PrintNodeOptions::default(),
            )
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            corsa::CorsaError::Unsupported(message) if message.contains("printNode is disabled by default")
        ));
        client.close().await.unwrap();
    });
}

#[test]
fn async_api_callbacks_work() {
    block_on(async {
        let client = ApiClient::spawn(
            support::api_config(ApiMode::AsyncJsonRpcStdio)
                .with_filesystem(support::virtual_fs(&[("/virtual/tsconfig.json", "{}")])),
        )
        .await
        .unwrap();
        let config = client
            .parse_config_file("/virtual/tsconfig.json")
            .await
            .unwrap();
        assert_eq!(config.options["virtual"], json!(true));
        client.close().await.unwrap();
    });
}

#[test]
fn concurrent_initialize_and_capability_calls_share_one_wire_request() {
    let count_dir = tempfile::tempdir().unwrap();
    block_on(async {
        let client = ApiClient::spawn(
            support::api_config(ApiMode::AsyncJsonRpcStdio)
                .with_env(
                    "CORSA_MOCK_COUNT_DIR",
                    count_dir.path().display().to_string(),
                )
                .with_env("CORSA_MOCK_DELAY_MS", "25"),
        )
        .await
        .unwrap();

        let initialize_workers = (0..8)
            .map(|_| {
                let client = client.clone();
                thread::spawn(move || block_on(async move { client.initialize().await.unwrap() }))
            })
            .collect::<Vec<_>>();
        for worker in initialize_workers {
            worker.join().unwrap();
        }

        let capability_workers = (0..8)
            .map(|_| {
                let client = client.clone();
                thread::spawn(move || {
                    block_on(async move { client.describe_capabilities().await.unwrap() })
                })
            })
            .collect::<Vec<_>>();
        for worker in capability_workers {
            worker.join().unwrap();
        }

        client.close().await.unwrap();
    });

    assert_eq!(count_lines(count_dir.path().join("initialize.count")), 1);
    assert_eq!(
        count_lines(count_dir.path().join("describeCapabilities.count")),
        1
    );
}

#[test]
fn dropping_many_snapshots_uses_bounded_release_queue() {
    let count_dir = tempfile::tempdir().unwrap();
    block_on(async {
        let client = ApiClient::spawn(
            support::api_config(ApiMode::AsyncJsonRpcStdio)
                .with_env(
                    "CORSA_MOCK_COUNT_DIR",
                    count_dir.path().display().to_string(),
                )
                // This case asserts the release queue drains all 64 snapshots;
                // the request timeout only has to be long enough that spawning
                // the mock under a loaded test runner does not trip it. A
                // one-second budget made the case flake on `initialize`.
                .with_request_timeout(Some(Duration::from_secs(30)))
                .with_shutdown_timeout(Duration::from_secs(30))
                .with_release_queue_capacity(1),
        )
        .await
        .unwrap();
        for index in 0..64 {
            let snapshot = client
                .update_snapshot(UpdateSnapshotParams {
                    open_project: Some(format!("/workspace/{index}/tsconfig.json")),
                    file_changes: None,
                    overlay_changes: None,
                })
                .await
                .unwrap();
            drop(snapshot);
        }
        client.close().await.unwrap();
    });
    assert_eq!(count_lines(count_dir.path().join("release.count")), 64);
}

#[test]
fn failed_explicit_snapshot_release_is_retried_on_drop() {
    let count_dir = tempfile::tempdir().unwrap();
    block_on(async {
        let client = ApiClient::spawn(
            support::api_config(ApiMode::AsyncJsonRpcStdio)
                .with_env(
                    "CORSA_MOCK_COUNT_DIR",
                    count_dir.path().display().to_string(),
                )
                .with_env("CORSA_MOCK_FAIL_RELEASE_ONCE", "1")
                .with_shutdown_timeout(Duration::from_secs(10)),
        )
        .await
        .unwrap();
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some("/workspace/tsconfig.json".into()),
                file_changes: None,
                overlay_changes: None,
            })
            .await
            .unwrap();

        let error = snapshot.release().await.unwrap_err();
        assert!(matches!(
            error,
            corsa::CorsaError::Rpc(error) if error.message.contains("mock release failure")
        ));

        drop(snapshot);
        client.close().await.unwrap();
    });
    assert_eq!(count_lines(count_dir.path().join("release.count")), 2);
}

#[test]
fn async_api_full_surface_methods() {
    block_on(async {
        let client = ApiClient::spawn(support::api_config(ApiMode::AsyncJsonRpcStdio))
            .await
            .unwrap();
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some("/workspace/tsconfig.json".into()),
                file_changes: None,
                overlay_changes: None,
            })
            .await
            .unwrap();
        let project = snapshot.projects[0].id.clone();
        let node = corsa::api::NodeHandle("1.3.80./workspace/src/index.ts".into());
        let symbol = client
            .get_symbol_at_location(snapshot.handle.clone(), project.clone(), node.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            client
                .get_symbols_at_locations(
                    snapshot.handle.clone(),
                    project.clone(),
                    vec![node.clone()]
                )
                .await
                .unwrap()[0]
                .as_ref()
                .unwrap()
                .id
                .as_str(),
            symbol.id.as_str()
        );
        assert_eq!(
            client
                .get_symbols_at_positions(
                    snapshot.handle.clone(),
                    project.clone(),
                    "/workspace/src/index.ts",
                    vec![1, 2]
                )
                .await
                .unwrap()
                .len(),
            2
        );
        assert!(
            client
                .get_declared_type_of_symbol(
                    snapshot.handle.clone(),
                    project.clone(),
                    symbol.id.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .resolve_name(
                    snapshot.handle.clone(),
                    project.clone(),
                    "value",
                    2,
                    Some(node.clone()),
                    None,
                    None,
                    None
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_parent_of_symbol_in_project(
                    snapshot.handle.clone(),
                    project.clone(),
                    symbol.id.clone(),
                )
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            client
                .get_members_of_symbol_in_project(
                    snapshot.handle.clone(),
                    project.clone(),
                    symbol.id.clone(),
                )
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            client
                .get_exports_of_symbol_in_project(
                    snapshot.handle.clone(),
                    project.clone(),
                    symbol.id.clone(),
                )
                .await
                .unwrap()
                .len(),
            1
        );
        let exported = client
            .get_export_symbol_of_symbol_in_project(
                snapshot.handle.clone(),
                project.clone(),
                symbol.id.clone(),
            )
            .await
            .unwrap();
        assert_eq!(exported.name, "exported");
        let ty = client
            .get_type_at_location(snapshot.handle.clone(), project.clone(), node.clone())
            .await
            .unwrap()
            .unwrap();
        assert!(
            client
                .get_type_at_locations(snapshot.handle.clone(), project.clone(), vec![node.clone()])
                .await
                .unwrap()[0]
                .is_some()
        );
        assert!(
            client
                .get_type_at_position(
                    snapshot.handle.clone(),
                    project.clone(),
                    "/workspace/src/index.ts",
                    1
                )
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            client
                .get_types_at_positions(
                    snapshot.handle.clone(),
                    project.clone(),
                    "/workspace/src/index.ts",
                    vec![1, 2]
                )
                .await
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            client
                .get_signatures_of_type(snapshot.handle.clone(), project.clone(), ty.id.clone(), 0)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            client
                .get_contextual_type(snapshot.handle.clone(), project.clone(), node.clone())
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_base_type_of_literal_type(
                    snapshot.handle.clone(),
                    project.clone(),
                    ty.id.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_shorthand_assignment_value_symbol(
                    snapshot.handle.clone(),
                    project.clone(),
                    node.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_type_of_symbol_at_location(
                    snapshot.handle.clone(),
                    project.clone(),
                    symbol.id.clone(),
                    node.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            client
                .type_to_type_node(
                    snapshot.handle.clone(),
                    project.clone(),
                    ty.id.clone(),
                    Some(node.clone()),
                    None
                )
                .await
                .unwrap()
                .unwrap()
                .as_bytes(),
            b"type-node"
        );
        assert!(
            client
                .is_context_sensitive(snapshot.handle.clone(), project.clone(), node.clone())
                .await
                .unwrap()
        );
        assert!(
            client
                .get_any_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_string_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_number_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_boolean_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_void_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_undefined_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_null_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_never_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_unknown_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_big_int_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        assert!(
            client
                .get_es_symbol_type(snapshot.handle.clone(), project.clone())
                .await
                .is_ok()
        );
        client.close().await.unwrap();
    });
}

#[test]
fn get_type_arguments_degrades_stale_handle_to_empty() {
    block_on(async {
        let client = ApiClient::spawn(support::api_config(ApiMode::AsyncJsonRpcStdio))
            .await
            .unwrap();
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some("/workspace/tsconfig.json".into()),
                file_changes: None,
                overlay_changes: None,
            })
            .await
            .unwrap();
        let project = snapshot.projects[0].id.clone();

        // A live handle returns the mock's normal single-element response.
        let live = client
            .get_type_arguments(
                snapshot.handle.clone(),
                project.clone(),
                corsa::api::TypeHandle::from("t0000000000000001"),
            )
            .await
            .unwrap();
        assert_eq!(live.len(), 1);

        // The mock reports this sentinel handle as evicted from its snapshot
        // registry; the client must degrade that to "no type arguments"
        // (regression for GH#211) instead of surfacing the error.
        let stale = client
            .get_type_arguments(
                snapshot.handle.clone(),
                project.clone(),
                corsa::api::TypeHandle::from("t00000000000000ff"),
            )
            .await
            .unwrap();
        assert!(stale.is_empty());

        client.close().await.unwrap();
    });
}

#[test]
fn get_type_arguments_hydrates_symbol_backed_arguments() {
    block_on(async {
        let client = ApiClient::spawn(support::api_config(ApiMode::AsyncJsonRpcStdio))
            .await
            .unwrap();
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some("/workspace/tsconfig.json".into()),
                file_changes: None,
                overlay_changes: None,
            })
            .await
            .unwrap();
        let project = snapshot.projects[0].id.clone();

        let arguments = client
            .get_type_arguments(
                snapshot.handle.clone(),
                project,
                corsa::api::TypeHandle::from("t00000000000000ee"),
            )
            .await
            .unwrap();

        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0].id.as_str(), "t00000000000000e2");
        assert_eq!(arguments[0].texts, ["Dog"]);

        client.close().await.unwrap();
    });
}

fn count_lines(path: impl AsRef<std::path::Path>) -> usize {
    fs::read_to_string(path)
        .map(|text| text.lines().count())
        .unwrap_or(0)
}

#[test]
fn async_api_supports_capabilities_overlay_diagnostics_and_editor_surface() {
    block_on(async {
        let client = ApiClient::spawn(support::api_config(ApiMode::AsyncJsonRpcStdio))
            .await
            .unwrap();
        let capabilities = client.describe_capabilities().await.unwrap();
        assert_eq!(capabilities.runtime.kind.as_deref(), Some("mock-corsa"));
        assert!(capabilities.overlay.update_snapshot_overlay_changes);
        assert!(capabilities.diagnostics.file);
        assert!(capabilities.editor.hover);
        assert!(capabilities.lsp.available);
        assert!(!capabilities.lsp.editor.hover);

        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some("/workspace/tsconfig.json".into()),
                file_changes: None,
                overlay_changes: Some(OverlayChanges {
                    upsert: vec![OverlayUpdate {
                        document: DocumentIdentifier::Uri {
                            uri: "corsa://overlay/demo.ts".into(),
                        },
                        text: "const value = 1;".into(),
                        version: Some(3),
                        language_id: Some("typescript".into()),
                    }],
                    delete: Vec::new(),
                }),
            })
            .await
            .unwrap();
        let project = snapshot.projects[0].id.clone();
        let changed_files = &snapshot
            .changes
            .as_ref()
            .unwrap()
            .changed_projects
            .get(&project)
            .unwrap()
            .changed_files;
        assert!(
            changed_files
                .iter()
                .any(|file| file == "corsa://overlay/demo.ts")
        );

        let file_diagnostics = client
            .get_diagnostics_for_file(
                snapshot.handle.clone(),
                project.clone(),
                "/workspace/src/index.ts",
            )
            .await
            .unwrap();
        assert_eq!(file_diagnostics.syntactic.len(), 1);
        assert_eq!(file_diagnostics.semantic.len(), 1);
        assert_eq!(file_diagnostics.suggestion.len(), 1);

        let project_diagnostics = client
            .get_diagnostics_for_project(snapshot.handle.clone(), project.clone())
            .await
            .unwrap();
        assert_eq!(project_diagnostics.files.len(), 1);

        let snapshot_diagnostics = client
            .get_diagnostics_for_snapshot(snapshot.handle.clone())
            .await
            .unwrap();
        assert_eq!(snapshot_diagnostics.projects.len(), 1);

        let hover = client
            .get_hover_at_position(
                snapshot.handle.clone(),
                project.clone(),
                "/workspace/src/index.ts",
                1,
            )
            .await
            .unwrap()
            .unwrap();
        assert!(
            serde_json::to_value(&hover).unwrap()["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("value")
        );

        let definition = client
            .get_definition_at_position(
                snapshot.handle.clone(),
                project.clone(),
                "/workspace/src/index.ts",
                1,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(&definition)
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );

        let references = client
            .get_references_at_position(
                snapshot.handle.clone(),
                project.clone(),
                "/workspace/src/index.ts",
                1,
            )
            .await
            .unwrap();
        assert_eq!(references.len(), 2);

        let rename = client
            .get_rename_at_position(
                snapshot.handle.clone(),
                project.clone(),
                "/workspace/src/index.ts",
                1,
                "renamedValue",
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(&rename).unwrap()["changes"]["file:///workspace/src/index.ts"][0]
                ["newText"],
            json!("renamedValue")
        );

        let completion = client
            .get_completion_at_position(
                snapshot.handle.clone(),
                project,
                "/workspace/src/index.ts",
                1,
                None,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(&completion).unwrap()["items"][0]["label"],
            json!("value")
        );

        client.close().await.unwrap();
    });
}

/// Exercises the checker endpoints the client exposes for type-aware tooling.
///
/// These answer questions the binding used to decide by parsing rendered type
/// text or re-reading source: assignability, array/tuple shape, apparent and
/// non-nullable forms, alias identity, and a signature's own parameters.
#[test]
fn async_api_exposes_checker_endpoints_for_type_aware_tooling() {
    block_on(async {
        let client = ApiClient::spawn(support::api_config(ApiMode::AsyncJsonRpcStdio))
            .await
            .unwrap();
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some("/workspace/tsconfig.json".into()),
                file_changes: None,
                overlay_changes: None,
            })
            .await
            .unwrap();
        let project = snapshot.projects[0].id.clone();
        let node = corsa::api::NodeHandle("1.3.80./workspace/src/index.ts".into());
        let ty = client
            .get_type_at_location(snapshot.handle.clone(), project.clone(), node.clone())
            .await
            .unwrap()
            .unwrap();
        let symbol = client
            .get_symbol_at_location(snapshot.handle.clone(), project.clone(), node.clone())
            .await
            .unwrap()
            .unwrap();
        let signature = client
            .get_signatures_of_type(snapshot.handle.clone(), project.clone(), ty.id.clone(), 0)
            .await
            .unwrap()
            .remove(0);

        // Checker predicates, not type-text inspection.
        assert!(
            client
                .is_type_assignable_to(
                    snapshot.handle.clone(),
                    project.clone(),
                    ty.id.clone(),
                    ty.id.clone()
                )
                .await
                .unwrap()
        );
        for predicate in ["is_array_type", "is_tuple_type", "is_array_like_type"] {
            let answered = match predicate {
                "is_array_type" => {
                    client
                        .is_array_type(snapshot.handle.clone(), project.clone(), ty.id.clone())
                        .await
                }
                "is_tuple_type" => {
                    client
                        .is_tuple_type(snapshot.handle.clone(), project.clone(), ty.id.clone())
                        .await
                }
                _ => {
                    client
                        .is_array_like_type(snapshot.handle.clone(), project.clone(), ty.id.clone())
                        .await
                }
            };
            assert!(answered.unwrap(), "{predicate} should reach the checker");
        }

        // Type normalization.
        for normalized in [
            client
                .get_apparent_type(snapshot.handle.clone(), project.clone(), ty.id.clone())
                .await
                .unwrap(),
            client
                .get_non_nullable_type(snapshot.handle.clone(), project.clone(), ty.id.clone())
                .await
                .unwrap(),
            client
                .get_widened_type(snapshot.handle.clone(), project.clone(), ty.id.clone())
                .await
                .unwrap(),
            client
                .get_base_constraint_of_type(
                    snapshot.handle.clone(),
                    project.clone(),
                    ty.id.clone(),
                )
                .await
                .unwrap(),
            client
                .get_fresh_type_of_type(snapshot.handle.clone(), project.clone(), ty.id.clone())
                .await
                .unwrap(),
            client
                .get_regular_type_of_type(snapshot.handle.clone(), project.clone(), ty.id.clone())
                .await
                .unwrap(),
            client
                .get_true_type_of_conditional_type(
                    snapshot.handle.clone(),
                    project.clone(),
                    ty.id.clone(),
                )
                .await
                .unwrap(),
            client
                .get_false_type_of_conditional_type(
                    snapshot.handle.clone(),
                    project.clone(),
                    ty.id.clone(),
                )
                .await
                .unwrap(),
            client
                .get_type_from_type_node(snapshot.handle.clone(), project.clone(), node.clone())
                .await
                .unwrap(),
        ] {
            assert!(normalized.is_some());
        }

        // Signature structure straight from the checker.
        assert_eq!(
            client
                .get_parameters_of_signature(
                    snapshot.handle.clone(),
                    project.clone(),
                    signature.id.clone()
                )
                .await
                .unwrap()
                .iter()
                .map(|parameter| parameter.name.as_str())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        assert!(
            client
                .get_this_parameter_of_signature(
                    snapshot.handle.clone(),
                    project.clone(),
                    signature.id.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            client
                .get_type_parameters_of_signature(
                    snapshot.handle.clone(),
                    project.clone(),
                    signature.id.clone()
                )
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            client
                .get_parameter_type(
                    snapshot.handle.clone(),
                    project.clone(),
                    signature.id.clone(),
                    0
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_target_of_signature(
                    snapshot.handle.clone(),
                    project.clone(),
                    signature.id.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_resolved_signature(snapshot.handle.clone(), project.clone(), node.clone())
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_signature_from_declaration(
                    snapshot.handle.clone(),
                    project.clone(),
                    node.clone()
                )
                .await
                .unwrap()
                .is_some()
        );

        // Alias and module identity.
        assert!(
            client
                .get_alias_symbol_of_type(snapshot.handle.clone(), project.clone(), ty.id.clone())
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            client
                .get_alias_type_arguments_of_type(
                    snapshot.handle.clone(),
                    project.clone(),
                    ty.id.clone()
                )
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            client
                .get_aliased_symbol(snapshot.handle.clone(), project.clone(), symbol.id.clone())
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_immediate_aliased_symbol(
                    snapshot.handle.clone(),
                    project.clone(),
                    symbol.id.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            client
                .get_exports_of_module(snapshot.handle.clone(), project.clone(), symbol.id.clone())
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            client
                .get_member_in_module_exports(
                    snapshot.handle.clone(),
                    project.clone(),
                    symbol.id.clone(),
                    "value"
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_export_specifier_local_target_symbol(
                    snapshot.handle.clone(),
                    project.clone(),
                    node.clone()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            client
                .get_property_of_type(
                    snapshot.handle.clone(),
                    project.clone(),
                    ty.id.clone(),
                    "value"
                )
                .await
                .unwrap()
                .is_some()
        );

        // Documentation and constants.
        assert_eq!(
            client
                .get_js_doc_tags(snapshot.handle.clone(), project.clone(), symbol.id.clone())
                .await
                .unwrap()[0]
                .name,
            "deprecated"
        );
        assert_eq!(
            client
                .get_documentation_comment(
                    snapshot.handle.clone(),
                    project.clone(),
                    symbol.id.clone()
                )
                .await
                .unwrap(),
            "docs"
        );
        assert_eq!(
            client
                .get_constant_value(snapshot.handle.clone(), project.clone(), node.clone())
                .await
                .unwrap(),
            Some(serde_json::json!(42))
        );
        assert_eq!(
            client
                .get_well_known_symbols(snapshot.handle.clone(), project.clone())
                .await
                .unwrap()
                .undefined
                .as_str(),
            "s0000000000000002"
        );

        client.close().await.unwrap();
    });
}
