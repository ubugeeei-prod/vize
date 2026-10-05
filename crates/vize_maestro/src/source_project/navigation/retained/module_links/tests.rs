//! Authored source-only laws: execution is reserved for the protected action.
#![expect(clippy::disallowed_types, reason = "laws compare actual snapshot Arcs")]

mod capacity;
mod jsx;
mod lifecycle;
mod ownership;
mod refusals;
mod sources;

use super::{Admission, ModuleOperandRefusal, RetainedNavigation, collect};
use crate::source_project::{
    SnapshotRefusal, SourceQueryProject, SourceSnapshotCache,
    navigation::{NativeNavigationProject, NavigationRefusal, worker::NavigationWorker},
};
use crate::{document::DocumentStore, runtime::block_on};
use std::sync::Arc;
use tower_lsp::lsp_types::{Position, Range, Url};
use vize_l0::{Allocator, SourceRoot};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::lang::js::{FileProducer, ModuleSourceErrorKind, ProgramInput, ProgramScope};

fn uri() -> Url {
    Url::parse("file:///module-operands.ts").unwrap()
}

fn range(start: (u32, u32), end: (u32, u32)) -> Range {
    Range::new(Position::new(start.0, start.1), Position::new(end.0, end.1))
}

fn worker(project: &NativeNavigationProject<'_>) -> Arc<NavigationWorker> {
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    project.worker(Arc::clone(query.snapshot())).unwrap()
}

fn with_original(
    source: &str,
    action: impl FnOnce(&RetainedNavigation<'_, '_>, &Admission<'_, '_, '_>),
) {
    let documents = DocumentStore::new();
    documents.open(uri(), source.into(), 1, "javascript".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&documents, &uri())
        .unwrap();
    let arena = Allocator::default();
    let block = SourceRoot::new(snapshot.source()).unwrap().whole_block();
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(snapshot.source(), block.span()).unwrap(),
        ProgramOptions::module(Lang::Js),
    );
    let mut producer = FileProducer::new(&arena, snapshot.source()).unwrap();
    producer
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 0).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    let lines = vize_l0::line_index::LineBreaks::Lsp
        .line_starts(snapshot.source())
        .collect::<Vec<_>>();
    {
        let admission = super::admit(&syntax, block, &file);
        let original = syntax.admitted_program().unwrap();
        let query = RetainedNavigation {
            snapshot: &snapshot,
            file: &file,
            lines: &lines,
            native: None,
            original: super::super::Original {
                program: original.program().body.as_ptr() as usize,
                statements: original.program().body.len(),
                sfc: None,
            },
        };
        action(&query, &admission);
    }
    drop(file);
    drop(syntax);
}
