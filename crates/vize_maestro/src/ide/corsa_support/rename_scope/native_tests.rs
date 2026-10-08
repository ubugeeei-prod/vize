//! Request-local projection ownership across the complete transaction.
#![expect(clippy::disallowed_macros, reason = "adversarial fixture construction")]

use tower_lsp::lsp_types::{
    CreateFile, DeleteFile, DocumentChangeOperation, DocumentChanges,
    OptionalVersionedTextDocumentIdentifier, RenameFile, ResourceOp, TextDocumentEdit, Url,
};

use super::{
    RenameScope,
    tests::{SOURCE, entry},
};
use crate::ide::{
    IdeContext,
    corsa_support::{
        canonical::CanonicalMaterializedSource,
        canonical_dependency_tests::{host_document, mapped_document},
    },
};
use crate::server::ServerState;

#[test]
fn native_scope_retains_closed_sfc_owners_through_the_final_packet() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Host.vue");
    let closed = root.path().join("Closed.vue");
    std::fs::write(&host, SOURCE).unwrap();
    std::fs::write(&closed, SOURCE).unwrap();
    let uri = Url::from_file_path(&host).unwrap();
    let closed_uri = Url::from_file_path(&closed).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let mut document = host_document(&uri, SOURCE);
    document
        .dependencies
        .push(mapped_document(&closed_uri, SOURCE));
    let mut raw = entry(Url::parse(&document.request_uri).unwrap());
    raw.changes.as_mut().unwrap().extend(
        entry(Url::parse(&document.dependencies[0].request_uri).unwrap())
            .changes
            .unwrap(),
    );
    let original = raw.clone();
    let mut scope = RenameScope::native(&ctx);
    assert!(scope.admits_native(&document, &raw));
    let mut final_edit = entry(uri.clone());
    final_edit
        .changes
        .as_mut()
        .unwrap()
        .extend(entry(closed_uri.clone()).changes.unwrap());
    assert!(scope.admits_authored(&final_edit));
    assert_eq!(raw, original);
    assert!(state.documents.get(&closed_uri).is_none());
    assert_eq!(std::fs::read_to_string(closed).unwrap(), SOURCE);
}

#[test]
fn native_scope_refuses_unknown_in_root_members_in_every_raw_and_final_container() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Host.vue");
    let asset = root.path().join("runtime-distribution/lib.dom.d.ts");
    std::fs::create_dir_all(asset.parent().unwrap()).unwrap();
    std::fs::write(&host, SOURCE).unwrap();
    let asset_bytes = b"interface GlobalEventHandlersEventMap { change: Event }";
    std::fs::write(&asset, asset_bytes).unwrap();
    let uri = Url::from_file_path(&host).unwrap();
    let asset_uri = Url::from_file_path(&asset).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let document = host_document(&uri, SOURCE);
    let native = entry(Url::parse(&document.request_uri).unwrap());
    let mut scope = RenameScope::native(&ctx);
    assert!(scope.admits_native(&document, &native));
    let mut mixed = native.clone();
    mixed
        .changes
        .as_mut()
        .unwrap()
        .extend(entry(asset_uri.clone()).changes.unwrap());
    assert!(!scope.admits_native(&document, &mixed));
    let mut final_edit = entry(uri.clone());
    final_edit
        .changes
        .as_mut()
        .unwrap()
        .extend(entry(asset_uri.clone()).changes.unwrap());
    assert!(!scope.admits_authored(&final_edit));
    let edits = entry(asset_uri.clone())
        .changes
        .unwrap()
        .remove(&asset_uri)
        .unwrap();
    let document_edit = TextDocumentEdit {
        text_document: OptionalVersionedTextDocumentIdentifier {
            uri: asset_uri.clone(),
            version: None,
        },
        edits: edits
            .into_iter()
            .map(tower_lsp::lsp_types::OneOf::Left)
            .collect(),
    };
    let mut mixed = native.clone();
    mixed.document_changes = Some(DocumentChanges::Edits(vec![document_edit.clone()]));
    assert!(!scope.admits_native(&document, &mixed));
    assert!(!scope.admits_authored(&mixed));
    for operation in [
        DocumentChangeOperation::Edit(document_edit),
        DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
            uri: asset_uri.clone(),
            options: None,
            annotation_id: None,
        })),
        DocumentChangeOperation::Op(ResourceOp::Delete(DeleteFile {
            uri: asset_uri.clone(),
            options: None,
        })),
        DocumentChangeOperation::Op(ResourceOp::Rename(RenameFile {
            old_uri: uri.clone(),
            new_uri: asset_uri.clone(),
            options: None,
            annotation_id: None,
        })),
        DocumentChangeOperation::Op(ResourceOp::Rename(RenameFile {
            old_uri: asset_uri.clone(),
            new_uri: uri.clone(),
            options: None,
            annotation_id: None,
        })),
    ] {
        let mut packet = native.clone();
        packet.document_changes = Some(DocumentChanges::Operations(vec![operation]));
        let unchanged = packet.clone();
        assert!(!scope.admits_native(&document, &packet));
        assert!(!scope.admits_authored(&packet));
        assert_eq!(packet, unchanged);
    }
    assert_eq!(std::fs::read(asset).unwrap(), asset_bytes);
    assert_eq!(std::fs::read_to_string(host).unwrap(), SOURCE);
}

#[test]
fn plain_host_and_materialized_identity_never_promote_a_rooted_runtime_asset() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Host.vue");
    let asset = root.path().join("runtime-distribution/lib.dom.d.ts");
    let private = root.path().join("private-project/asset.d.ts");
    std::fs::create_dir_all(asset.parent().unwrap()).unwrap();
    std::fs::create_dir_all(private.parent().unwrap()).unwrap();
    std::fs::write(&host, SOURCE).unwrap();
    std::fs::write(&asset, "interface Events { change: Event }").unwrap();
    std::fs::write(&private, "interface Events { change: Event }").unwrap();
    let uri = Url::from_file_path(&host).unwrap();
    let asset_uri = Url::from_file_path(&asset).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let mut document = host_document(&uri, SOURCE);
    document
        .materialized_sources
        .push(CanonicalMaterializedSource {
            source_uri: asset_uri.clone(),
            source: "interface Events { change: Event }".into(),
            request_uri: Url::from_file_path(&private).unwrap().to_string().into(),
            virtual_result: host_document(&uri, SOURCE).virtual_result,
            mapping_kind: vize_canon::CorsaMaterializedMappingKind::AuthoredIdentity,
        });
    for kind in [
        vize_canon::CorsaMaterializedMappingKind::AuthoredIdentity,
        vize_canon::CorsaMaterializedMappingKind::Generated,
        vize_canon::CorsaMaterializedMappingKind::Synthetic,
    ] {
        document.materialized_sources[0].mapping_kind = kind;
        let mut mixed = entry(Url::parse(&document.request_uri).unwrap());
        mixed.changes.as_mut().unwrap().extend(
            entry(Url::from_file_path(&private).unwrap())
                .changes
                .unwrap(),
        );
        let mut scope = RenameScope::native(&ctx);
        assert!(!scope.admits_native(&document, &mixed));
        assert!(!scope.admits_authored(&entry(asset_uri.clone())));
    }
    // Even a populated generic host projection is not an SFC producer role.
    document.source_uri = asset_uri;
    let mut scope = RenameScope::native(&ctx);
    assert!(!scope.admits_native(
        &document,
        &entry(Url::parse(&document.request_uri).unwrap())
    ));
}

#[cfg(unix)]
#[test]
fn vue_named_physical_alias_cannot_authorize_a_runtime_declaration() {
    let root = tempfile::tempdir().unwrap();
    let asset = root.path().join("lib.dom.d.ts");
    let alias = root.path().join("Alias.vue");
    std::fs::write(&asset, "interface Events { change: Event }").unwrap();
    std::os::unix::fs::symlink(&asset, &alias).unwrap();
    let uri = Url::from_file_path(&alias).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    state
        .documents
        .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, SOURCE);
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let document = host_document(&uri, SOURCE);
    let mut scope = RenameScope::native(&ctx);
    assert!(!scope.admits_native(
        &document,
        &entry(Url::parse(&document.request_uri).unwrap())
    ));
    assert!(!scope.admits_authored(&entry(uri.clone())));
    assert_eq!(
        std::fs::read_to_string(asset).unwrap(),
        "interface Events { change: Event }"
    );
}

#[cfg(unix)]
#[test]
fn vue_named_hardlink_cannot_turn_a_native_asset_into_an_authored_source() {
    let root = tempfile::tempdir().unwrap();
    let asset = root.path().join("lib.dom.d.ts");
    let alias = root.path().join("Alias.vue");
    std::fs::write(&asset, "interface Events { change: Event }").unwrap();
    std::fs::hard_link(&asset, &alias).unwrap();
    let uri = Url::from_file_path(&alias).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    state
        .documents
        .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, SOURCE);
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let document = host_document(&uri, SOURCE);
    let mut scope = RenameScope::native(&ctx);
    assert!(!scope.admits_native(
        &document,
        &entry(Url::parse(&document.request_uri).unwrap())
    ));
    assert!(!scope.admits_authored(&entry(uri.clone())));
}

#[cfg(unix)]
#[test]
fn generated_request_links_cannot_promote_native_asset_uris() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Host.vue");
    let asset = root.path().join("lib.dom.d.ts");
    std::fs::write(&host, SOURCE).unwrap();
    std::fs::write(&asset, "interface Events { change: Event }").unwrap();
    let uri = Url::from_file_path(&host).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    state
        .documents
        .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, SOURCE);
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let document = host_document(&uri, SOURCE);
    let request = Url::parse(&document.request_uri)
        .unwrap()
        .to_file_path()
        .unwrap();
    for hardlink in [false, true] {
        if hardlink {
            std::fs::hard_link(&asset, &request).unwrap();
        } else {
            std::os::unix::fs::symlink(&asset, &request).unwrap();
        }
        for raw_uri in [
            Url::from_file_path(&asset).unwrap(),
            Url::from_file_path(&request).unwrap(),
        ] {
            let edit = entry(raw_uri);
            let mut scope = RenameScope::native(&ctx);
            assert!(!scope.admits_native(&document, &edit));
            assert!(!scope.admits_virtual_native(&uri, &document.request_uri, &edit));
        }
        std::fs::remove_file(&request).unwrap();
    }
    assert_eq!(
        std::fs::read_to_string(asset).unwrap(),
        "interface Events { change: Event }"
    );
}

#[cfg(unix)]
#[test]
fn retargeted_authored_alias_cannot_reuse_the_final_path_memo() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let source = root.path().join("Source.vue");
    let foreign = outside.path().join("Foreign.vue");
    let alias = root.path().join("Alias.vue");
    std::fs::write(&source, SOURCE).unwrap();
    std::fs::write(&foreign, SOURCE).unwrap();
    std::os::unix::fs::symlink(&source, &alias).unwrap();
    let uri = Url::from_file_path(&alias).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.path().to_path_buf());
    state
        .documents
        .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, SOURCE);
    let ctx = IdeContext::testing(&state, &uri, 24, SOURCE.to_string());
    let document = host_document(&uri, SOURCE);
    let mut scope = RenameScope::native(&ctx);
    assert!(scope.admits_native(
        &document,
        &entry(Url::parse(&document.request_uri).unwrap())
    ));
    assert!(scope.admits_authored(&entry(uri.clone())));
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&foreign, &alias).unwrap();
    assert!(!scope.admits_authored(&entry(uri.clone())));
    assert_eq!(std::fs::read_to_string(foreign).unwrap(), SOURCE);
}
