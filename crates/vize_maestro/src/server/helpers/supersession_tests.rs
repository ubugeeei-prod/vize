//! Supersession of the importer diagnostics fan-out (#3315).
//!
//! Every case asserts the full ordered list of importers actually refreshed, so
//! a pass that quietly refreshes one of two — or refreshes in a different order
//! — fails instead of passing on a subset match.

use tower_lsp::LspService;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use super::MaestroServer;

const LEAF: &str = "<template>\n  <div>leaf</div>\n</template>\n";

fn importer_source(leaf: &str) -> String {
    vize_s0::cstr!(
        "<script setup lang=\"ts\">\nimport Leaf from './{leaf}'\n</script>\n\
         <template>\n  <Leaf />\n</template>\n"
    )
    .to_string()
}

/// A server with two open Vue documents importing `Leaf.vue`, which is itself
/// open at version 1. Type checking stays off so the fan-out is pure bookkeeping
/// and the test needs no Corsa backend.
struct Fixture {
    _dir: tempfile::TempDir,
    service: LspService<MaestroServer>,
    leaf: Url,
    importers: Vec<Url>,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let leaf_path = dir.path().join("Leaf.vue");
    std::fs::write(&leaf_path, LEAF).unwrap();
    let leaf = Url::from_file_path(&leaf_path).unwrap();

    let (service, _socket) = LspService::new(MaestroServer::new);
    service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({
            "lint": false,
            "typecheck": false,
            "ecosystem": false
        })));
    let state = &service.inner().state;
    state
        .documents
        .open(leaf.clone(), LEAF.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&leaf, LEAF);

    let mut importers = Vec::new();
    for name in ["Alpha.vue", "Beta.vue"] {
        let path = dir.path().join(name);
        let source = importer_source("Leaf.vue");
        std::fs::write(&path, &source).unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        state
            .documents
            .open(uri.clone(), source.clone(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, &source);
        importers.push(uri);
    }
    importers.sort();

    Fixture {
        _dir: dir,
        service,
        leaf,
        importers,
    }
}

fn refresh(fixture: &Fixture, version: i32) -> Vec<Url> {
    crate::runtime::block_on(
        fixture
            .service
            .inner()
            .publish_importer_diagnostics(&fixture.leaf, version),
    )
}

fn bump_leaf(fixture: &Fixture, version: i32) {
    fixture.service.inner().state.documents.apply_changes(
        &fixture.leaf,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: LEAF.to_string(),
        }],
        version,
    );
}

#[test]
fn current_version_refreshes_every_open_importer() {
    let fixture = fixture();
    assert_eq!(refresh(&fixture, 1), fixture.importers);
}

#[test]
fn superseded_pass_refreshes_nothing() {
    let fixture = fixture();
    bump_leaf(&fixture, 2);

    // The pass for version 1 is dead work: version 2's own pass redoes this
    // fan-out from the text the user actually has. Publishing nothing here is
    // also what keeps stale diagnostics off screen — the pass stops before
    // publishing, not after computing.
    assert_eq!(refresh(&fixture, 1), Vec::<Url>::new());
}

#[test]
fn the_newest_version_still_refreshes_after_superseded_passes() {
    let fixture = fixture();
    bump_leaf(&fixture, 2);
    bump_leaf(&fixture, 3);

    assert_eq!(refresh(&fixture, 1), Vec::<Url>::new());
    assert_eq!(refresh(&fixture, 2), Vec::<Url>::new());
    assert_eq!(refresh(&fixture, 3), fixture.importers);
}

#[test]
fn a_closed_document_refreshes_nothing() {
    let fixture = fixture();
    fixture.service.inner().state.documents.close(&fixture.leaf);

    assert_eq!(refresh(&fixture, 1), Vec::<Url>::new());
}
