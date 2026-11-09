//! Genuine native differential law; ordinary explicit native-disable carries no credit.

use super::CorsaProjectClient;
use crate::{batch::virtual_project::MaterializedFileDelta, file_uri::path_to_file_uri};
use level_test_support::original_diagnosing_process as control;
use std::{fs, io::Write, path::Path};
use vize_l0::{FxHashMap, cstr};

fn state(client: &CorsaProjectClient) -> serde_json::Value {
    serde_json::json!({
        "cwd":client.cwd,"projectRoot":client.project_root,
        "explicitConfig":client.explicit_project_config,
        "authored":client.authored_workspace,
        "logical":client.logical_workspace_overlays,
        "logicalReset":client.logical_api_reset_pending,
        "materialized":client.materialized_project_session,
        "preference":client.materialized_project_preference,
        "overlayApiDisabled":client.overlay_api_disabled,
        "sharedRequested":client.shared_editor_transport_requested,
        "shared":client.shared_editor_transport,
        "attached":client.project_session_attached_to_editor,
        "hasSession":client.session.is_some(),"hasEditor":client.editor_lsp.is_some(),
        "hasOriginalDiagnosing":client.original_diagnosing_session.is_some(),
        "willRenameSupported":client.editor_lsp_will_rename_supported,
        "capabilities":client.capabilities.as_ref(),
        "dirty":client.editor_lsp_documents_dirty,
        "closed":client.closed,
        "texts":client.document_texts,"versions":client.overlay_versions,
        "sessionUris":client.session_document_uris,"externalUris":client.external_document_uris,
        "diagnostics":client.diagnostics,
    })
}

fn report(
    client: &mut CorsaProjectClient,
    root: &Path,
    source: &str,
) -> Result<serde_json::Value, vize_l0::String> {
    let uri = path_to_file_uri(&root.join("source.ts"));
    let documents = FxHashMap::from_iter([(uri.clone(), source.into())]);
    let report = client.diagnostics_via_editor_lsp(&uri, &documents)?;
    serde_json::to_value(report).map_err(|error| cstr!("Cannot encode native report: {error}"))
}

fn stage(direct: bool, phase: &str, value: serde_json::Value) {
    let mut stdout = std::io::stdout();
    stdout
        .write_all(
            cstr!(
                "FIRST_CANON_FACTORY_STAGE {}\n",
                serde_json::json!({"direct":direct,"phase":phase,"value":value})
            )
            .as_bytes(),
        )
        .unwrap();
    stdout.flush().unwrap();
}

#[test]
fn native_first_materialized_editor_matches_existing_activation_and_delta() {
    if !control::enabled() {
        return;
    }
    let fixture = control::fixture();
    let authored = tempfile::tempdir().unwrap();
    let canon = tempfile::tempdir().unwrap();
    let authored_root = authored.path().canonicalize().unwrap();
    let canon_root = canon.path().canonicalize().unwrap();
    control::write_fixture(&authored_root, &fixture);
    control::write_fixture(&canon_root, &fixture);
    let authored_dependencies = authored_root.join("node_modules");
    fs::create_dir_all(authored_dependencies.join("vue")).unwrap();
    // This is the generated Canon configuration shape. The diagnosing editor
    // must attest this exact root's config, rather than the authored root.
    let config = canon_root.join("tsconfig.json");
    let first_delta = MaterializedFileDelta {
        changed: vec![config.clone(), config.clone()],
        created: vec![canon_root.join("source.ts"), canon_root.join("shared.js")],
        deleted: vec![canon_root.join("node_modules/retired")],
        topology_changed: true,
    };
    let native = control::runtime();
    let mut packets = Vec::new();
    for direct in [false, true] {
        fs::write(
            canon_root.join("source.ts"),
            fixture["source"].as_str().unwrap(),
        )
        .unwrap();
        fs::write(
            canon_root.join("shared.js"),
            fixture["shared"].as_str().unwrap(),
        )
        .unwrap();
        fs::write(&config, serde_json::to_vec(&fixture["config"]).unwrap()).unwrap();
        let constructor = if direct {
            CorsaProjectClient::new_for_editor_materialized_workspace(
                native.to_str(),
                &authored_root,
                &canon_root,
                &config,
                &first_delta,
            )
        } else {
            CorsaProjectClient::new_for_editor_workspace(native.to_str(), &authored_root).and_then(
                |mut client| {
                    let activation = client.synchronize_materialized_project(
                        &canon_root,
                        Some(&config),
                        &first_delta,
                    );
                    stage(
                        direct,
                        "oldActivation",
                        serde_json::json!({"result":&activation,"state":state(&client)}),
                    );
                    activation.map(|()| client)
                },
            )
        };
        stage(
            direct,
            "constructor",
            match &constructor {
                Ok(client) => serde_json::json!({"ok":true,"state":state(client)}),
                Err(error) => serde_json::json!({"ok":false,"error":error}),
            },
        );
        let mut client = constructor.unwrap();
        let initial = state(&client);
        let storage = client.overlay_storage.clone();
        let dependencies = storage.root().parent().unwrap().join("node_modules");
        assert_eq!(
            dependencies.canonicalize().unwrap(),
            authored_dependencies.canonicalize().unwrap()
        );
        for ancestor in storage.root().ancestors() {
            assert_ne!(ancestor, authored_root.as_path());
            assert_ne!(ancestor, canon_root.as_path());
        }
        let source = fixture["source"].as_str().unwrap();
        let bad = report(&mut client, &canon_root, source);
        stage(
            direct,
            "firstQuery",
            serde_json::json!({"result":&bad,"state":state(&client)}),
        );
        let bad = bad.unwrap();
        let bad_view = client
            .diagnosing_view_for_test(&path_to_file_uri(&canon_root.join("source.ts")))
            .unwrap();
        // An independent editor is legitimately lazy until the first report.
        let initial_owner = control::native_lsp(&canon_root, &native);
        let uri = path_to_file_uri(&canon_root.join("source.ts"));
        let diagnosing_project = client.diagnosing_project_info_for_test(&uri);
        stage(
            direct,
            "diagnosingProject",
            serde_json::json!({"result":&diagnosing_project,"owner":{
                "pid":initial_owner.pid,"birth":initial_owner.birth,
                "executable":initial_owner.executable,
            }}),
        );
        let diagnosing_project = diagnosing_project.unwrap();
        assert_eq!(
            diagnosing_project
                .get("configFilePath")
                .and_then(serde_json::Value::as_str),
            config.to_str()
        );
        assert_eq!(control::native_lsp(&canon_root, &native), initial_owner);
        let edited = fixture["leafEdit"].as_str().unwrap();
        client.did_change(&uri, edited).unwrap();
        let repaired = report(&mut client, &canon_root, edited);
        stage(
            direct,
            "repairedQuery",
            serde_json::json!({"result":&repaired}),
        );
        let repaired = repaired.unwrap();
        let repaired_view = client.diagnosing_view_for_test(&uri).unwrap();
        let owner = control::native_lsp(&canon_root, &native);
        assert_eq!(owner, initial_owner);
        if owner != initial_owner {
            control::assert_reaped(&initial_owner);
        }
        let before_empty = state(&client);
        client
            .synchronize_materialized_project(
                &canon_root,
                Some(&config),
                &MaterializedFileDelta::default(),
            )
            .unwrap();
        let after_empty = state(&client);
        assert!(std::sync::Arc::ptr_eq(&storage, &client.overlay_storage));
        assert_eq!(control::native_lsp(&canon_root, &native), owner);
        let shared = canon_root.join("shared.js");
        fs::remove_file(&shared).unwrap();
        client
            .refresh_materialized_files(&[], &[], std::slice::from_ref(&shared))
            .unwrap();
        let missing = report(&mut client, &canon_root, edited).unwrap();
        let missing_view = client.diagnosing_view_for_test(&uri).unwrap();
        fs::write(&shared, fixture["shared"].as_str().unwrap()).unwrap();
        client
            .refresh_materialized_files(&[], &[shared], &[])
            .unwrap();
        let restored = report(&mut client, &canon_root, edited).unwrap();
        let restored_view = client.diagnosing_view_for_test(&uri).unwrap();
        assert_eq!(control::native_lsp(&canon_root, &native), owner);
        let api_eligible = client.can_use_api_for_uri(&uri);
        client.shutdown().unwrap();
        control::assert_reaped(&owner);
        let packet = serde_json::json!({
            "initial":initial,"bad":bad,"repaired":repaired,
            "diagnosingProject":diagnosing_project,
            "beforeEmpty":before_empty,"afterEmpty":after_empty,
            "badView":bad_view,"repairedView":repaired_view,"missingView":missing_view,"restoredView":restored_view,
            "privateConfig":fs::read(storage.root().join("tsconfig.json")).ok(),
            "missing":missing,"restored":restored,
            "apiEligibleForEdited":api_eligible,
            "canonSource":fs::read(canon_root.join("source.ts")).unwrap(),
            "canonShared":fs::read(canon_root.join("shared.js")).unwrap(),
            "canonConfig":fs::read(&config).unwrap(),
        });
        // Retain the complete old/new result before payload equality assertions.
        std::io::stdout()
            .write_all(
                cstr!(
                    "INITIAL_CANON_PROJECT {}\n",
                    serde_json::json!({
                        "direct":direct,"native":native,"packet":packet,
                        "owner":{"pid":owner.pid,"birth":owner.birth,"executable":owner.executable},
                        "config":config,"authoredRoot":authored_root,"canonRoot":canon_root,
                    })
                )
                .as_bytes(),
            )
            .unwrap();
        assert_eq!(packet["bad"], fixture["expected"]);
        assert_eq!(packet["repaired"], fixture["editedExpected"]);
        assert_eq!(packet["privateConfig"], serde_json::Value::Null);
        assert_eq!(packet["apiEligibleForEdited"], false);
        assert_eq!(packet["afterEmpty"]["logical"], true);
        assert_eq!(packet["afterEmpty"]["attached"], true);
        for (field, text) in [
            ("badView", source),
            ("repairedView", edited),
            ("missingView", edited),
            ("restoredView", edited),
        ] {
            let view = &packet[field];
            assert_eq!(view["uri"], uri.as_str());
            assert_eq!(view["config"], packet["diagnosingProject"]);
            assert_eq!(
                view["documents"],
                serde_json::to_value(FxHashMap::from_iter([(
                    uri.clone(),
                    vize_l0::String::from(text)
                )]))
                .unwrap()
            );
            assert_eq!(view["acknowledgedGeneration"], view["writtenGeneration"]);
            assert_eq!(view["dirtyDocuments"], serde_json::json!([]));
        }
        assert!(
            packet["repairedView"]["writtenGeneration"]
                .as_u64()
                .unwrap()
                > packet["badView"]["writtenGeneration"].as_u64().unwrap()
        );
        assert_eq!(
            packet["missing"],
            serde_json::json!({
                "kind":"full", "items":[{
                    "code":2307,
                    "message":"Cannot find module './shared.js' or its corresponding type declarations.",
                    "range":{"start":{"line":0,"character":22},"end":{"line":0,"character":35}},
                    "severity":1,"source":"ts",
                }]
            })
        );
        assert_eq!(packet["restored"], fixture["editedExpected"]);
        assert_eq!(packet["beforeEmpty"], packet["afterEmpty"]);
        packets.push(packet);
    }
    let old = packets.first().unwrap();
    let direct = packets.last().unwrap();
    // Post-query identity maps retain each owner's distinct private storage
    // names above. Compare those unchanged within each owner, and compare the
    // complete first-activation state and whole authored diagnostic results.
    for field in [
        "initial",
        "diagnosingProject",
        "bad",
        "repaired",
        "missing",
        "restored",
        "canonSource",
        "canonShared",
        "canonConfig",
    ] {
        assert_eq!(old[field], direct[field], "first-project field {field}");
    }
    assert_eq!(
        fs::read(authored_root.join("source.ts")).unwrap(),
        fixture["source"].as_str().unwrap().as_bytes()
    );
    assert_eq!(
        fs::read(authored_root.join("shared.js")).unwrap(),
        fixture["shared"].as_str().unwrap().as_bytes()
    );
}
