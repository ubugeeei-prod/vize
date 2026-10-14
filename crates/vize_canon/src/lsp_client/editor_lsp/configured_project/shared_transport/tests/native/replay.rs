//! Disk invalidation cannot turn retained unsaved buffers into disk answers.

use super::{CorsaProjectClient, control, fs, only_owner};

#[test]
fn native_editor_resolves_package_alias_from_monorepo_root_replays_unsaved_api_after_disk_events() {
    if !control::enabled() {
        return;
    }
    let fixture = control::fixture();
    let native = control::runtime();
    let root = tempfile::tempdir().unwrap();
    control::write_fixture(root.path(), &fixture);
    let source = fixture["source"].as_str().unwrap();
    let edited = fixture["leafEdit"].as_str().unwrap();
    let uri = crate::file_uri::path_to_file_uri(&root.path().join("source.ts"));
    let sibling = root.path().join("other.ts");
    fs::write(&sibling, edited).unwrap();
    let sibling_uri = crate::file_uri::path_to_file_uri(&sibling);
    let mut client =
        CorsaProjectClient::new_for_editor_workspace(native.to_str(), root.path()).unwrap();
    client.did_change(&uri, edited).unwrap();
    client
        .did_open_batch_fast(&[(&sibling_uri, source)])
        .unwrap();
    assert_eq!(
        api_type(&mut client, root.path(), edited).as_str(),
        "() => number"
    );
    assert_eq!(
        sibling_type(&mut client, &sibling_uri, source).as_str(),
        "() => string"
    );
    let retained = client.document_texts.clone();
    let mut owner = only_owner(root.path(), &native);
    let declaration = root.path().join("ambient.d.ts");
    for change in ["configuration", "declaration"] {
        if change == "configuration" {
            let mut config = fixture["config"].clone();
            config["compilerOptions"]["noUnusedLocals"] = true.into();
            fs::write(
                root.path().join("tsconfig.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
        } else {
            fs::write(
                &declaration,
                "declare const changedDeclaration: unique symbol;\n",
            )
            .unwrap();
        }
        client.invalidate_disk_project_state().unwrap();
        control::assert_reaped(&owner);
        assert!(client.session.is_none());
        assert!(client.editor_lsp.is_none());
        assert_eq!(client.document_texts, retained);
        // No editor request or changed buffer precedes the first API answer.
        // Disk contains the opposite types for both complete retained buffers.
        assert_eq!(
            api_type(&mut client, root.path(), edited).as_str(),
            "() => number"
        );
        assert_eq!(
            sibling_type(&mut client, &sibling_uri, source).as_str(),
            "() => string"
        );
        assert_eq!(client.document_texts, retained);
        assert!(client.project_session_attached_to_editor);
        let replacement = only_owner(root.path(), &native);
        assert_ne!(replacement, owner);
        client
            .did_open_batch_fast(&[(&uri, edited), (&sibling_uri, source)])
            .unwrap();
        assert_eq!(
            api_type(&mut client, root.path(), edited).as_str(),
            "() => number"
        );
        assert_eq!(
            sibling_type(&mut client, &sibling_uri, source).as_str(),
            "() => string"
        );
        assert_eq!(only_owner(root.path(), &native), replacement);
        assert_eq!(
            fs::read(root.path().join("source.ts")).unwrap(),
            source.as_bytes()
        );
        assert_eq!(fs::read(&sibling).unwrap(), edited.as_bytes());
        owner = replacement;
    }
    client.shutdown().unwrap();
    control::assert_reaped(&owner);
    assert_eq!(
        fs::read(root.path().join("source.ts")).unwrap(),
        source.as_bytes()
    );
    assert_eq!(fs::read(&sibling).unwrap(), edited.as_bytes());
}

fn api_type(
    client: &mut CorsaProjectClient,
    root: &std::path::Path,
    source: &str,
) -> vize_l0::String {
    sibling_type(
        client,
        &crate::file_uri::path_to_file_uri(&root.join("source.ts")),
        source,
    )
}

fn sibling_type(client: &mut CorsaProjectClient, uri: &str, source: &str) -> vize_l0::String {
    client.project_session().unwrap();
    let session_uri = client.session_document_uris.get(uri).unwrap().clone();
    let session = client.project_session().unwrap();
    let ty = corsa::runtime::block_on(session.get_type_at_position(
        crate::lsp_client::session::uri_document_identifier(&session_uri),
        source.find("answer").unwrap() as u32,
    ))
    .unwrap()
    .unwrap();
    corsa::runtime::block_on(session.type_to_string(ty.id, None, None))
        .unwrap()
        .into()
}
