//! Genuine native differential law; ordinary explicit native-disable carries no credit.

use super::CorsaProjectClient;
use crate::{batch::virtual_project::MaterializedFileDelta, file_uri::path_to_file_uri};
use davinci_test_support::original_diagnosing_process as control;
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

fn report(client: &mut CorsaProjectClient, root: &Path, source: &str) -> serde_json::Value {
    let uri = path_to_file_uri(&root.join("source.ts"));
    let documents = FxHashMap::from_iter([(uri.clone(), source.into())]);
    serde_json::to_value(client.diagnostics_via_editor_lsp(&uri, &documents).unwrap()).unwrap()
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
    // A distinct selected config proves the direct bootstrap did not substitute
    // its authored root or merely infer the default Canon configuration.
    let config = canon_root.join("tsconfig.selected.json");
    fs::rename(canon_root.join("tsconfig.json"), &config).unwrap();
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
        let mut client = if direct {
            CorsaProjectClient::new_for_editor_materialized_workspace(
                native.to_str(),
                &authored_root,
                &canon_root,
                &config,
                &first_delta,
            )
            .unwrap()
        } else {
            let mut client =
                CorsaProjectClient::new_for_editor_workspace(native.to_str(), &authored_root)
                    .unwrap();
            client
                .synchronize_materialized_project(&canon_root, Some(&config), &first_delta)
                .unwrap();
            client
        };
        let initial = state(&client);
        assert!(client.project_session_attached_to_editor);
        assert_eq!(
            Path::new(&client.project_session().unwrap().project().config_file_name),
            config
        );
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
        let initial_owner = control::native_lsp(&canon_root, &native);
        let source = fixture["source"].as_str().unwrap();
        let bad = report(&mut client, &canon_root, source);
        let edited = fixture["leafEdit"].as_str().unwrap();
        let uri = path_to_file_uri(&canon_root.join("source.ts"));
        client.did_change(&uri, edited).unwrap();
        let repaired = report(&mut client, &canon_root, edited);
        let owner = control::native_lsp(&canon_root, &native);
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
        client.shutdown().unwrap();
        control::assert_reaped(&owner);
        let packet = serde_json::json!({
            "initial":initial,"bad":bad,"repaired":repaired,
            "beforeEmpty":before_empty,"afterEmpty":after_empty,
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
        "bad",
        "repaired",
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
