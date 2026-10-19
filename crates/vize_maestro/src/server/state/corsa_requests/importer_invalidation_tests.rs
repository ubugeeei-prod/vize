//! Cross-package declaration refresh invalidates each actual native owner once.

use std::{path::Path, sync::Arc, sync::atomic::Ordering};

use tower_lsp::{LspService, lsp_types::Url};

use super::super::super::MaestroServer;
use super::ServerState;

struct Fixture {
    _root: tempfile::TempDir,
    service: LspService<MaestroServer>,
    declaration: Url,
    importers: Vec<Url>,
    owners: Vec<Arc<ServerState>>,
}

fn file(root: &Path, name: &str, source: &str) -> Url {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, source).unwrap();
    Url::from_file_path(path).unwrap()
}

fn fixture() -> Fixture {
    fixture_with_declaration("vendor/types.d.mts")
}

fn fixture_with_declaration(declaration_name: &str) -> Fixture {
    let root = crate::test_support::tempdir().unwrap();
    file(
        root.path(),
        "vite.config.mjs",
        r#"export default { vue: {"languageServer":{"lint":false,"typecheck":false}} };"#,
    );
    for package in ["vendor", "left", "right", "idle"] {
        file(root.path(), &vize_l0::cstr!("{package}/package.json"), "{}");
    }
    let declaration = file(root.path(), declaration_name, "export interface Kind {}\n");
    let (service, _socket) = LspService::new(MaestroServer::new);
    let server = service.inner();
    server.state.set_workspace_root(root.path().to_path_buf());
    server.state.load_workspace_config(root.path());
    server
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({
            "lint":false,"typecheck":false,"ecosystem":false
        })));
    let mut importers = Vec::new();
    let direct_source = vize_l0::cstr!(
        "<script setup lang=\"ts\">import type {{ Kind }} from './{declaration_name}'; let item: Kind</script><template />"
    )
    .to_string();
    for (name, source) in [
        ("Alpha.vue", direct_source.as_str()),
        ("Beta.vue", "<template />"),
        ("left/App.vue", "<template />"),
        ("right/App.vue", "<template />"),
    ] {
        let uri = file(root.path(), name, source);
        server
            .state
            .documents
            .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
        let project = crate::runtime::block_on(server.for_document(&uri));
        project.state.update_virtual_docs(&uri, source);
        importers.push(uri);
    }
    server.state.documents.open(
        declaration.clone(),
        "export interface Kind {}\n".to_owned(),
        1,
        "typescript".to_owned(),
    );
    let vendor = file(root.path(), "vendor/Closed.vue", "<template />");
    let mut owners = vec![server.state.clone()];
    for uri in [&importers[2], &importers[3], &vendor] {
        owners.push(
            crate::runtime::block_on(server.for_document(uri))
                .state
                .clone(),
        );
    }
    let idle = file(root.path(), "idle/Closed.vue", "<template />");
    owners.push(
        crate::runtime::block_on(server.for_document(&idle))
            .state
            .clone(),
    );
    importers.sort();
    Fixture {
        _root: root,
        service,
        declaration,
        importers,
        owners,
    }
}

fn revisions(fixture: &Fixture) -> Vec<u64> {
    fixture
        .owners
        .iter()
        .map(|owner| owner.corsa_environment_revision.load(Ordering::Acquire))
        .collect()
}

#[test]
fn declaration_fanout_marks_actual_owners_once_and_preserves_idle_owners() {
    let fixture = fixture();
    assert_eq!(fixture.owners.len(), 5);
    for (index, owner) in fixture.owners.iter().enumerate() {
        for other in &fixture.owners[index + 1..] {
            assert!(!Arc::ptr_eq(owner, other));
        }
    }
    // Alpha is present in both the explicit reverse index and the declaration
    // fan-out, while Alpha and Beta share the same native owner.
    assert_eq!(
        fixture.owners[0].open_importers(&fixture.declaration),
        [fixture.importers[0].clone()]
    );
    let before = revisions(&fixture);
    let server =
        crate::runtime::block_on(fixture.service.inner().for_document(&fixture.declaration));
    let refreshed = crate::runtime::block_on(
        server.publish_importer_diagnostics(&fixture.declaration, Some(1)),
    );
    assert_eq!(refreshed, fixture.importers);
    assert_eq!(
        revisions(&fixture),
        [
            before[0] + 2,
            before[1] + 2,
            before[2] + 2,
            before[3],
            before[4]
        ]
    );
    assert_eq!(
        fixture
            .owners
            .iter()
            .map(|owner| owner.has_corsa_bridge())
            .collect::<Vec<_>>(),
        [false; 5]
    );
}

#[test]
fn a_preinvalidated_same_owner_is_not_marked_again() {
    let fixture = fixture_with_declaration("types.d.mts");
    let server =
        crate::runtime::block_on(fixture.service.inner().for_document(&fixture.declaration));
    assert!(Arc::ptr_eq(&server.state, &fixture.owners[0]));
    crate::runtime::block_on(
        crate::server::workspace_files::invalidate_changed_document_disk_project_state(
            &server,
            &fixture.declaration,
        ),
    );
    let before = revisions(&fixture);
    assert_eq!(
        crate::runtime::block_on(server.publish_importer_diagnostics_with_current_owner(
            &fixture.declaration,
            Some(1),
            true,
        )),
        fixture.importers
    );
    assert_eq!(
        revisions(&fixture),
        [
            before[0],
            before[1] + 2,
            before[2] + 2,
            before[3],
            before[4]
        ]
    );
}

#[test]
fn a_superseded_cross_owner_declaration_refresh_marks_nothing() {
    let fixture = fixture();
    fixture.service.inner().state.documents.apply_changes(
        &fixture.declaration,
        vec![tower_lsp::lsp_types::TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "export interface Kind { changed: true }\n".to_owned(),
        }],
        2,
    );
    let before = revisions(&fixture);
    let server =
        crate::runtime::block_on(fixture.service.inner().for_document(&fixture.declaration));
    assert_eq!(
        crate::runtime::block_on(
            server.publish_importer_diagnostics(&fixture.declaration, Some(1))
        ),
        Vec::<Url>::new()
    );
    assert_eq!(revisions(&fixture), before);
}
