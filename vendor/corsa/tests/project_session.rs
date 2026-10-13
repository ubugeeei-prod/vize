mod support;

use corsa::{
    api::{ApiMode, OverlayChanges, OverlayUpdate, ProjectSession, TypeProbeOptions},
    fast::CompactString,
    runtime::block_on,
};

#[test]
fn project_session_reuses_snapshot_and_project_handles() {
    block_on(async {
        let mut session = ProjectSession::spawn(
            support::api_config(ApiMode::AsyncJsonRpcStdio),
            "/workspace/tsconfig.json",
            Some("/workspace/src/index.ts".into()),
        )
        .await
        .unwrap();

        let project_id = session.project().id.clone();
        let symbol = session
            .get_symbol_at_position("/workspace/src/index.ts", 1)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(symbol.name, "value");

        let type_response = session
            .get_type_of_symbol(symbol.id)
            .await
            .unwrap()
            .unwrap();
        let rendered = session
            .type_to_string(type_response.id, None, None)
            .await
            .unwrap();
        assert_eq!(rendered, "type:string");

        session.refresh(None).await.unwrap();
        assert_eq!(session.project().id, project_id);
        assert!(
            session
                .get_type_at_position("/workspace/src/index.ts", 1)
                .await
                .unwrap()
                .is_some()
        );

        session.close().await.unwrap();
    });
}

#[test]
fn project_session_builds_checker_probe_views() {
    block_on(async {
        let session = ProjectSession::spawn(
            support::api_config(ApiMode::AsyncJsonRpcStdio),
            "/workspace/tsconfig.json",
            Some("/workspace/src/index.ts".into()),
        )
        .await
        .unwrap();

        let probe = session
            .probe_type_at_position(
                "/workspace/src/index.ts",
                1,
                TypeProbeOptions {
                    load_property_types: true,
                    load_signatures: true,
                },
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(probe.type_texts, vec![CompactString::from("type-text")]);
        assert_eq!(probe.property_names, vec![CompactString::from("value")]);
        assert_eq!(
            probe.property_types,
            vec![vec![CompactString::from("type-text")]]
        );
        assert_eq!(
            probe.call_signatures,
            vec![vec![vec![CompactString::from("type-text")]]]
        );
        assert_eq!(
            probe.return_types,
            vec![vec![CompactString::from("type-text")]]
        );

        session.close().await.unwrap();
    });
}

#[test]
fn project_session_supports_capabilities_diagnostics_and_editor_helpers() {
    block_on(async {
        let mut session = ProjectSession::spawn(
            support::api_config(ApiMode::AsyncJsonRpcStdio),
            "/workspace/tsconfig.json",
            Some("/workspace/src/index.ts".into()),
        )
        .await
        .unwrap();

        let capabilities = session.describe_capabilities().await.unwrap();
        assert!(capabilities.editor.completion);
        assert!(capabilities.lsp.available);
        assert!(!capabilities.lsp.editor.completion);

        session
            .refresh_with_overlay_changes(
                None,
                Some(OverlayChanges {
                    upsert: vec![OverlayUpdate {
                        document: "/workspace/src/virtual.ts".into(),
                        text: "export const value = 1;".into(),
                        version: Some(2),
                        language_id: Some("typescript".into()),
                    }],
                    delete: Vec::new(),
                }),
            )
            .await
            .unwrap();

        let project_diagnostics = session.get_diagnostics_for_project().await.unwrap();
        assert_eq!(project_diagnostics.files.len(), 1);
        let file_diagnostics = session
            .get_diagnostics_for_file("/workspace/src/index.ts")
            .await
            .unwrap();
        assert_eq!(file_diagnostics.semantic.len(), 1);
        let snapshot_diagnostics = session.get_diagnostics_for_snapshot().await.unwrap();
        assert_eq!(snapshot_diagnostics.projects.len(), 1);

        let hover = session
            .get_hover_at_position("/workspace/src/index.ts", 1)
            .await
            .unwrap()
            .unwrap();
        assert!(
            serde_json::to_value(&hover).unwrap()["contents"]["value"]
                .as_str()
                .unwrap()
                .contains("value")
        );
        let definition = session
            .get_definition_at_position("/workspace/src/index.ts", 1)
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
        assert_eq!(
            session
                .get_references_at_position("/workspace/src/index.ts", 1)
                .await
                .unwrap()
                .len(),
            2
        );
        let rename = session
            .get_rename_at_position("/workspace/src/index.ts", 1, "renamedValue")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(&rename).unwrap()["changes"]["file:///workspace/src/index.ts"][0]
                ["newText"],
            serde_json::json!("renamedValue")
        );
        let completion = session
            .get_completion_at_position("/workspace/src/index.ts", 1, None)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(&completion).unwrap()["items"][0]["label"],
            serde_json::json!("value")
        );

        session.close().await.unwrap();
    });
}
