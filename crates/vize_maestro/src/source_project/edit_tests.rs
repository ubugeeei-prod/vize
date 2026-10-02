use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::edit::{EditError, EditSet, VersionedSource};
use vize_l1::embed::{EmbedSource, prepare_attribute_value};

use crate::document::DocumentStore;

use super::{SnapshotRefusal, SourceEditRefusal, SourceSnapshotCache};

fn open(documents: &DocumentStore, path: &str, source: &str) -> Result<Url, impl std::fmt::Debug> {
    Url::parse(path).map(|uri| {
        documents.open(uri.clone(), source.into(), 1, "vue".into());
        uri
    })
}

#[test]
fn actual_edit_model_borrows_the_original_cached_root_and_produces_only_new_text() {
    let documents = DocumentStore::new();
    let uri = open(&documents, "file:///edit.vue", "<div>😀</div>\r\n").unwrap();
    let cache = SourceSnapshotCache::default();
    let snapshot = cache.capture(&documents, &uri).unwrap();
    let frame = snapshot.versioned_source().unwrap();
    assert_eq!(frame.root().source().as_ptr(), snapshot.source().as_ptr());
    let edits = [frame.edit(Span::new(5, 9), "α").unwrap()];
    let set = EditSet::new(frame, &edits).unwrap();
    let result = snapshot.apply_edits(&documents, set).unwrap();
    assert_eq!(result.as_str(), "<div>α</div>\r\n");
    assert_eq!(documents.text(&uri).unwrap(), "<div>😀</div>\r\n");
    assert_eq!(snapshot.check_current(&documents), Ok(()));
}

#[test]
fn equal_copy_with_the_real_key_and_version_still_refuses_physical_source_mismatch() {
    let documents = DocumentStore::new();
    let uri = open(&documents, "file:///copy.vue", "<div>same</div>").unwrap();
    let snapshot = SourceSnapshotCache::default()
        .capture(&documents, &uri)
        .unwrap();
    let copied = vize_l0::String::from(snapshot.source());
    let foreign = VersionedSource::new(
        snapshot.key(),
        snapshot.version(),
        SourceRoot::new(&copied).unwrap(),
    );
    let edits = [foreign.edit(Span::new(5, 9), "changed").unwrap()];
    let set = EditSet::new(foreign, &edits).unwrap();
    assert_eq!(
        snapshot.apply_edits(&documents, set),
        Err(SourceEditRefusal::Edit(EditError::SourceMismatch))
    );
    let prepared = EmbedSource::authored(&copied, Span::new(5, 9)).unwrap();
    assert!(matches!(
        snapshot
            .versioned_source()
            .unwrap()
            .project_edit(prepared, Span::new(0, 4), "x"),
        Err(EditError::SourceMismatch)
    ));
}

#[test]
fn real_host_key_refuses_foreign_document_edits_despite_equal_versions_and_bytes() {
    let documents = DocumentStore::new();
    let first_uri = open(&documents, "file:///first.vue", "<div/>").unwrap();
    let second_uri = open(&documents, "file:///second.vue", "<div/>").unwrap();
    let cache = SourceSnapshotCache::default();
    let first = cache.capture(&documents, &first_uri).unwrap();
    let second = cache.capture(&documents, &second_uri).unwrap();
    let foreign = second.versioned_source().unwrap();
    let edits = [foreign.edit(Span::new(1, 4), "span").unwrap()];
    let set = EditSet::new(foreign, &edits).unwrap();
    assert_eq!(
        first.apply_edits(&documents, set),
        Err(SourceEditRefusal::Edit(EditError::SnapshotMismatch))
    );
}

#[test]
fn close_reopen_with_same_client_version_cannot_apply_retained_native_edits() {
    let documents = DocumentStore::new();
    let uri = open(&documents, "file:///reopen.vue", "<div/>").unwrap();
    let snapshot = SourceSnapshotCache::default()
        .capture(&documents, &uri)
        .unwrap();
    let frame = snapshot.versioned_source().unwrap();
    let edits = [frame.edit(Span::new(1, 4), "span").unwrap()];
    let set = EditSet::new(frame, &edits).unwrap();
    documents.close(&uri);
    assert_eq!(
        snapshot.apply_edits(&documents, set),
        Err(SourceEditRefusal::Host(SnapshotRefusal::MissingDocument))
    );
    documents.open(uri, "<div/>".into(), 1, "vue".into());
    assert_eq!(
        snapshot.apply_edits(&documents, set),
        Err(SourceEditRefusal::Host(SnapshotRefusal::Superseded))
    );
}

#[test]
fn actual_current_revision_change_refuses_edit_output_before_emission() {
    let documents = DocumentStore::new();
    let uri = open(&documents, "file:///superseded.vue", "<div/>").unwrap();
    let snapshot = SourceSnapshotCache::default()
        .capture(&documents, &uri)
        .unwrap();
    let frame = snapshot.versioned_source().unwrap();
    let edits = [frame.edit(Span::new(1, 4), "span").unwrap()];
    let set = EditSet::new(frame, &edits).unwrap();
    assert!(documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "<div/>".into()
        }],
        2
    ));
    assert_eq!(
        snapshot.apply_edits(&documents, set),
        Err(SourceEditRefusal::Host(SnapshotRefusal::Superseded))
    );
}

#[test]
fn genuine_decoded_embed_projects_to_the_same_retained_authored_root() {
    let documents = DocumentStore::new();
    let uri = open(&documents, "file:///entity.vue", "xx&fjlig;yy").unwrap();
    let snapshot = SourceSnapshotCache::default()
        .capture(&documents, &uri)
        .unwrap();
    let arena = Allocator::default();
    let prepared = prepare_attribute_value(&arena, snapshot.source(), Span::new(2, 9)).unwrap();
    let frame = snapshot.versioned_source().unwrap();
    assert!(matches!(
        frame.project_edit(prepared, Span::new(0, 1), "x"),
        Err(EditError::Projection(_))
    ));
    let edits = [frame.project_edit(prepared, Span::new(0, 2), "ok").unwrap()];
    let set = EditSet::new(frame, &edits).unwrap();
    assert_eq!(
        snapshot.apply_edits(&documents, set).unwrap().as_str(),
        "xxokyy"
    );
    assert_eq!(snapshot.source(), "xx&fjlig;yy");
}
