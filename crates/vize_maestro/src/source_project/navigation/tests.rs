#![expect(
    clippy::disallowed_types,
    reason = "tests compare actual immutable host snapshot Arc identities"
)]
mod capacity;
mod jsx;
mod lifecycle;
mod refusals;
mod retention;
mod vue;

use super::{NativeNavigationProject, NavigationRefusal, SourceQueryProject, coordinates};
use crate::{document::DocumentStore, runtime::block_on};
use std::sync::Arc;
use tower_lsp::lsp_types::{Location, Position, Range, Url};

fn uri() -> Url {
    Url::parse("file:///native.ts").unwrap()
}
fn location(start: (u32, u32), end: (u32, u32)) -> Location {
    Location::new(
        uri(),
        Range::new(Position::new(start.0, start.1), Position::new(end.0, end.1)),
    )
}

fn cached(project: &NativeNavigationProject<'_>) -> Arc<super::worker::NavigationWorker> {
    Arc::clone(
        project
            .workers
            .lock()
            .get(&uri())
            .unwrap()
            .result
            .as_ref()
            .unwrap(),
    )
}

#[test]
fn unicode_crlf_definitions_and_references_reuse_one_original_worker() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "/*😀*/ const café=1;\r\ncafé;\r\n".into(),
        1,
        "typescript".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(1, 1))),
        Ok(Some(location((0, 13), (0, 17))))
    );
    let original = cached(&project);
    assert_eq!(
        block_on(project.references(&uri(), Position::new(0, 14), true)),
        Ok(vec![location((0, 13), (0, 17)), location((1, 0), (1, 4))])
    );
    assert_eq!(
        block_on(project.references(&uri(), Position::new(1, 1), false)),
        Ok(vec![location((1, 0), (1, 4))])
    );
    assert!(Arc::ptr_eq(&original, &cached(&project)));
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(0, 0))),
        Ok(None)
    );
}

#[test]
fn same_spelling_shadowed_bindings_keep_distinct_real_file_targets() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "const value=1;\nfunction f(value){return value;}\nvalue;".into(),
        1,
        "javascript".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(1, 26))),
        Ok(Some(location((1, 11), (1, 16))))
    );
    assert_eq!(
        block_on(project.references(&uri(), Position::new(0, 7), true)),
        Ok(vec![location((0, 6), (0, 11)), location((2, 0), (2, 5))])
    );
    assert_eq!(
        block_on(project.references(&uri(), Position::new(1, 12), false)),
        Ok(vec![location((1, 25), (1, 30))])
    );
}

#[test]
fn imported_name_definition_is_local_import_binding_only() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "import { value as local } from './other';\nlocal;".into(),
        1,
        "javascript".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    assert_eq!(
        block_on(project.definition(&uri(), Position::new(1, 2))),
        Ok(Some(location((0, 18), (0, 23))))
    );
    assert_eq!(
        block_on(project.references(&uri(), Position::new(1, 2), false)),
        Ok(vec![location((1, 0), (1, 5))])
    );
}

#[test]
fn utf16_surrogate_interiors_and_crlf_spill_are_refused() {
    let source = "/*😀*/ const café=1;\r\ncafé;\rnext;";
    let lines = vize_l0::line_index::LineBreaks::Lsp
        .line_starts(source)
        .collect::<Vec<_>>();
    assert_eq!(
        coordinates::offset(source, &lines, Position::new(0, 3)),
        Err(NavigationRefusal::Position)
    );
    assert_eq!(
        coordinates::offset(source, &lines, Position::new(0, 99)),
        Err(NavigationRefusal::Position)
    );
    assert_eq!(
        coordinates::offset(source, &lines, Position::new(3, 0)),
        Err(NavigationRefusal::Position)
    );
    assert_eq!(
        coordinates::offset(source, &lines, Position::new(1, 6)),
        Err(NavigationRefusal::Position)
    );
    assert_eq!(
        coordinates::offset(source, &lines, Position::new(2, 0)),
        Ok(source.find("next").unwrap())
    );
    assert_eq!(
        coordinates::range(source, &lines, vize_l0::Span::new(3, 5)),
        Err(NavigationRefusal::Projection)
    );
}

#[test]
fn equal_foreign_snapshot_cannot_reuse_the_local_worker_or_file() {
    let documents = DocumentStore::new();
    let foreign = DocumentStore::new();
    for store in [&documents, &foreign] {
        store.open(uri(), "const value=1;value;".into(), 1, "javascript".into());
    }
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let (local, _) = project.source.begin_query(&uri()).unwrap();
    let worker = project.worker(Arc::clone(local.snapshot())).unwrap();
    let cache = crate::source_project::SourceSnapshotCache::default();
    let snapshot = cache.capture(&foreign, &uri()).unwrap();
    assert_eq!(snapshot.source(), local.snapshot().source());
    assert!(!worker.belongs_to(&snapshot));
    assert!(matches!(
        project.worker(snapshot),
        Err(NavigationRefusal::Host(
            crate::source_project::SnapshotRefusal::Superseded
        ))
    ));
    assert!(Arc::ptr_eq(&worker, &cached(&project)));
}
