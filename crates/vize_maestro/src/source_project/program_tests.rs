#![expect(
    clippy::disallowed_types,
    reason = "the actual host snapshot Arc remains alive while the genuine parser borrows its original buffer"
)]

use std::sync::Arc;

use tower_lsp::lsp_types::Url;
use vize_l0::{Allocator, Span};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{ProgramOptions, parse_program_once},
};

use crate::document::DocumentStore;
use crate::runtime::block_on;

use super::{SnapshotRefusal, SourceSnapshotCache};

#[test]
fn genuine_shared_program_observation_reads_the_same_once_captured_host_buffer() {
    let documents = DocumentStore::new();
    let uri = Url::parse("file:///native.ts").unwrap();
    let text = "/*\n native\n*/\nexport const value: number = 1;\n";
    documents.open(uri.clone(), text.into(), 3, "typescript".into());
    let cache = SourceSnapshotCache::default();
    let (query, _) = cache.begin_query(&documents, &uri).unwrap();
    let retained = Arc::clone(query.snapshot());
    let ready = block_on(query.run(&documents, |snapshot| async move {
        let arena = Allocator::default();
        let input = EmbedSource::authored(
            snapshot.source(),
            Span::new(0, snapshot.source().len() as u32),
        )
        .unwrap();
        // This is the actual source-owned parser invocation, not a helper tree.
        let parsed = parse_program_once(&arena, input, ProgramOptions::module(Lang::Ts));
        let admitted = parsed.admitted_program().unwrap();
        assert_eq!(admitted.source().as_ptr(), snapshot.source().as_ptr());
        assert_eq!(admitted.source(), text);
        assert_eq!(admitted.program().body.len(), 1);
        let spans = parsed
            .comments()
            .map(|comment| comment.authored_span().unwrap())
            .collect::<Vec<_>>();
        (snapshot.key().revision, snapshot.version(), spans)
    }))
    .unwrap();
    assert_eq!(
        ready.publish(&documents, |facts| facts),
        Ok((retained.revision(), 3, vec![Span::new(0, 13)]))
    );
    assert_eq!(
        retained.native_file(&documents),
        Err(SnapshotRefusal::NativeFileUnavailable)
    );
}

#[test]
fn parser_owned_recovery_observation_is_not_promoted_by_a_current_host_snapshot() {
    let documents = DocumentStore::new();
    let uri = Url::parse("file:///recovery.js").unwrap();
    documents.open(
        uri.clone(),
        "/* retained */ const value = /x/uv;".into(),
        1,
        "javascript".into(),
    );
    let cache = SourceSnapshotCache::default();
    let (query, _) = cache.begin_query(&documents, &uri).unwrap();
    let ready = block_on(query.run(&documents, |snapshot| async move {
        let arena = Allocator::default();
        let input = EmbedSource::authored(
            snapshot.source(),
            Span::new(0, snapshot.source().len() as u32),
        )
        .unwrap();
        let parsed = parse_program_once(&arena, input, ProgramOptions::module(Lang::Js));
        assert!(parsed.comments().next().is_some());
        assert!(parsed.diagnostics().next().is_some());
        parsed.admitted_program().is_some()
    }))
    .unwrap();
    assert_eq!(ready.publish(&documents, |admitted| admitted), Ok(false));
}
