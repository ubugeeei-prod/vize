//! Genuine native currentness refusal and independently held snapshot lifetime.

use crate::lsp_client::{CorsaProjectClient, session::uri_document_identifier};
use corsa::{
    api::{ApiClient, ManagedSnapshot, ProjectHandle},
    runtime::block_on,
};
use davinci_test_support::original_diagnosing_process as control;
use std::fs;
use vize_l0::String;

fn held_type(
    api: &ApiClient,
    snapshot: &ManagedSnapshot,
    project: &ProjectHandle,
    uri: &str,
    source: &str,
) -> String {
    let ty = block_on(api.get_type_at_position(
        snapshot.handle.clone(),
        project.clone(),
        uri_document_identifier(uri),
        source.find("answer").unwrap() as u32,
    ))
    .unwrap()
    .unwrap();
    block_on(api.type_to_string(snapshot.handle.clone(), project.clone(), ty.id, None, None))
        .unwrap()
        .into()
}

#[test]
fn native_logical_revision_refusal_quarantines_current_api_but_preserves_held_snapshot() {
    if !control::enabled() {
        return;
    }
    let fixture = control::fixture();
    let native = control::runtime();
    let root = tempfile::tempdir().unwrap();
    control::write_fixture(root.path(), &fixture);
    let uri = crate::file_uri::path_to_file_uri(&root.path().join("source.ts"));
    let mut client =
        CorsaProjectClient::new_for_editor_workspace(native.to_str(), root.path()).unwrap();
    let owner = control::native_lsp(root.path(), &native);
    let number = fixture["leafEdit"].as_str().unwrap();
    client.did_change(&uri, number).unwrap();
    assert!(!client.logical_api_reset_pending);
    let session = client.project_session().unwrap();
    let api = session.client().clone();
    let snapshot = session.snapshot_lease();
    let project = session.project().id.clone();
    assert_eq!(
        held_type(&api, &snapshot, &project, &uri, number).as_str(),
        "() => number",
    );
    let malformed = "untitled://unaccepted.ts";
    assert_eq!(
        client.did_open(malformed, "export const unaccepted = 1;"),
        Err("Logical Corsa workspace requires a file URI: untitled://unaccepted.ts".into()),
    );
    assert!(client.logical_api_reset_pending);
    assert_eq!(
        client.project_session().map(|_| ()),
        Err("Logical editor API snapshot does not own the current retained revision".into()),
    );
    assert_eq!(
        client.project_session_mut().map(|_| ()),
        Err("Logical editor API snapshot does not own the current retained revision".into()),
    );
    assert_eq!(control::native_lsp(root.path(), &native), owner,);
    client.did_close(malformed).unwrap();
    let boolean = "export function answer() { return true; }\n";
    client.did_change(&uri, boolean).unwrap();
    assert!(!client.logical_api_reset_pending);
    let current = client.project_session().unwrap();
    assert_eq!(
        held_type(
            current.client(),
            &current.snapshot_lease(),
            &current.project().id,
            &uri,
            boolean,
        )
        .as_str(),
        "() => boolean",
    );
    assert_eq!(
        held_type(&api, &snapshot, &project, &uri, number).as_str(),
        "() => number",
    );
    assert_eq!(control::native_lsp(root.path(), &native), owner);
    assert_eq!(
        fs::read(root.path().join("source.ts")).unwrap(),
        fixture["source"].as_str().unwrap().as_bytes(),
    );
    drop(snapshot);
    drop(api);
    client.shutdown().unwrap();
    control::assert_reaped(&owner);
}
