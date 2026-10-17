//! Deferred validation cannot manufacture a same-owner source mutation.

use std::path::Path;

use tower_lsp::{
    LspService,
    lsp_types::{DidOpenTextDocumentParams, TextDocumentContentChangeEvent, TextDocumentItem, Url},
};
use vize_l0::cstr;

use super::{MaestroServer, refresh_initial_importers};

const PLAIN: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/bare-script-symbol-routes/H/plain.ts.txt"
);
const DECLARATION: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/bare-script-symbol-routes/H/ordinary.d.ts.txt"
);

struct Fixture {
    _root: tempfile::TempDir,
    service: LspService<MaestroServer>,
    dependency: Url,
    source: &'static str,
    importers: Vec<Url>,
    owners: Vec<MaestroServer>,
}

fn file(root: &Path, name: &str, source: &str) -> Url {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, source).unwrap();
    Url::from_file_path(path).unwrap()
}

fn fixture(name: &str, source: &'static str, cross_owner: bool) -> Fixture {
    let root = tempfile::tempdir().unwrap();
    file(
        root.path(),
        "vite.config.mjs",
        "export default { vue: {} };",
    );
    if cross_owner {
        for package in ["vendor", "left", "idle"] {
            file(root.path(), &cstr!("{package}/package.json"), "{}");
        }
    }
    let dependency = file(root.path(), name, source);
    // Exercise the actual open fallback deterministically, then the worker's
    // exact fan-out entry. No wall-clock grace or native backend is required.
    let (service, _socket) = LspService::new(|client| {
        let mut server = MaestroServer::new(client);
        server.initial_diagnostics = None;
        server
    });
    let server = service.inner();
    server.state.set_workspace_root(root.path().to_path_buf());
    server.state.load_workspace_config(root.path());
    server
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({
            "lint":false,"typecheck":false,"ecosystem":false
        })));
    let mut importers = Vec::new();
    let mut owners = vec![server.with_project_state(server.state.clone())];
    for importer in ["Alpha.vue", "Beta.vue"]
        .into_iter()
        .chain(cross_owner.then_some("left/App.vue"))
    {
        let relative = if importer.starts_with("left/") {
            cstr!("../{name}")
        } else {
            cstr!("./{name}")
        };
        let content = cstr!("<script setup lang=\"ts\">import '{relative}';</script><template />");
        let uri = file(root.path(), importer, &content);
        server.state.documents.open(
            uri.clone(),
            content.as_str().to_owned(),
            1,
            "vue".to_owned(),
        );
        let project = crate::runtime::block_on(server.for_document(&uri));
        project.state.update_virtual_docs(&uri, &content);
        if !owners
            .iter()
            .any(|owner| std::ptr::eq(owner.state.as_ref(), project.state.as_ref()))
        {
            owners.push(server.with_project_state(project.state.clone()));
        }
        importers.push(uri);
    }
    if cross_owner {
        for package in ["vendor", "idle"] {
            let uri = file(root.path(), &cstr!("{package}/Closed.vue"), "<template />");
            let project = crate::runtime::block_on(server.for_document(&uri));
            owners.push(server.with_project_state(project.state.clone()));
        }
    }
    importers.sort();
    Fixture {
        _root: root,
        service,
        dependency,
        source,
        importers,
        owners,
    }
}

fn revisions(fixture: &Fixture) -> Vec<u64> {
    fixture
        .owners
        .iter()
        .map(|owner| {
            owner
                .state
                .corsa_request_stamp()
                .environment_revision_for_test()
        })
        .collect()
}

fn open(fixture: &Fixture) {
    crate::runtime::block_on(
        fixture
            .service
            .inner()
            .open_document(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: fixture.dependency.clone(),
                    language_id: "typescript".to_owned(),
                    version: 1,
                    text: fixture.source.to_owned(),
                },
            }),
    );
}

#[test]
fn prepared_open_and_late_same_owner_worker_preserve_a_stable_native_scope() {
    for (name, source) in [("plain.ts", PLAIN), ("ordinary.d.ts", DECLARATION)] {
        let fixture = fixture(name, source, false);
        let owner = &fixture.owners[0].state;
        let before = revisions(&fixture);
        let old = owner.corsa_request_stamp();
        open(&fixture);
        assert!(!old.is_current(owner));
        assert_eq!(revisions(&fixture), [before[0] + 2]);

        crate::runtime::block_on(async {
            let scope = owner.corsa_request_scope().await;
            assert!(scope.is_current());
            assert_eq!(
                refresh_initial_importers(fixture.service.inner(), &fixture.dependency, 1).await,
                fixture.importers
            );
            assert_eq!(revisions(&fixture), [before[0] + 2]);
            assert!(scope.is_current());
        });
        assert!(!owner.has_corsa_bridge());
    }
}

#[test]
fn a_real_cross_owner_declaration_edit_marks_each_importer_owner_once() {
    let fixture = fixture("vendor/ordinary.d.ts", DECLARATION, true);
    open(&fixture);
    let before = revisions(&fixture);
    let stamps = fixture
        .owners
        .iter()
        .map(|owner| owner.state.corsa_request_stamp())
        .collect::<Vec<_>>();
    crate::runtime::block_on(fixture.service.inner().apply_document_changes(
        &fixture.dependency,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: cstr!("// changed\n{}", fixture.source).as_str().to_owned(),
        }],
        2,
    ));
    assert_eq!(
        revisions(&fixture),
        [before[0] + 2, before[1] + 2, before[2] + 2, before[3]]
    );
    assert_eq!(
        stamps
            .iter()
            .zip(&fixture.owners)
            .map(|(stamp, owner)| stamp.is_current(&owner.state))
            .collect::<Vec<_>>(),
        [false; 4]
    );
    let owner = crate::runtime::block_on(fixture.service.inner().for_document(&fixture.dependency));
    let prepared = revisions(&fixture);
    assert_eq!(
        crate::runtime::block_on(refresh_initial_importers(&owner, &fixture.dependency, 2)),
        fixture.importers
    );
    assert_eq!(
        revisions(&fixture),
        [prepared[0] + 2, prepared[1] + 2, prepared[2], prepared[3]]
    );
    assert_eq!(
        fixture
            .owners
            .iter()
            .map(|owner| owner.state.has_corsa_bridge())
            .collect::<Vec<_>>(),
        [false; 4]
    );
}

#[test]
fn worker_supersession_and_active_authored_mutation_still_refuse_stale_scopes() {
    let fixture = fixture("plain.ts", PLAIN, false);
    open(&fixture);
    let owner = &fixture.owners[0].state;
    let stable = owner.corsa_request_stamp();
    let mutation = owner.documents.source_mutation_for_test();
    assert!(!stable.is_current(owner));
    let during = owner.corsa_request_stamp();
    drop(mutation);
    assert!(!stable.is_current(owner));
    assert!(!during.is_current(owner));

    let before = revisions(&fixture);
    owner.documents.apply_changes(
        &fixture.dependency,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: fixture.source.to_owned(),
        }],
        2,
    );
    let current = owner.corsa_request_stamp();
    assert_eq!(
        crate::runtime::block_on(refresh_initial_importers(
            fixture.service.inner(),
            &fixture.dependency,
            1
        )),
        Vec::<Url>::new()
    );
    assert_eq!(revisions(&fixture), before);
    assert!(current.is_current(owner));

    // The public native wrapper still refuses an actual active source-set
    // mutation. Its body needs no backend, so this remains deterministic.
    fixture
        .service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({"typecheck":true})));
    let mutation = owner.documents.source_mutation_for_test();
    assert_eq!(
        crate::runtime::block_on(fixture.service.inner().native_request(async { Ok(()) })),
        Err(tower_lsp::jsonrpc::Error::content_modified())
    );
    drop(mutation);
    assert_eq!(
        crate::runtime::block_on(fixture.service.inner().native_request(async { Ok(()) })),
        Ok(())
    );
    assert!(!owner.has_corsa_bridge());
}
