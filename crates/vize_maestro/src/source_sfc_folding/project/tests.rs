use std::task::{Context, Poll};

use futures::Future;
use futures::task::noop_waker_ref;
use tower_lsp::lsp_types::{
    FoldingRange, FoldingRangeKind, Position, Range, TextDocumentContentChangeEvent, Url,
};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};

use crate::document::DocumentStore;
use crate::runtime::block_on;
use crate::source_project::{SnapshotRefusal, SourceQueryProject};

use super::{SfcQueryRefusal, query_sfc_comments};

mod refusals;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

fn expected(start: u32, end: u32) -> FoldingRange {
    FoldingRange {
        start_line: start,
        start_character: None,
        end_line: end,
        end_character: None,
        kind: Some(FoldingRangeKind::Comment),
        collapsed_text: None,
    }
}

#[test]
fn real_project_query_projects_original_whole_file_nonzero_script_coordinates() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = Url::parse("file:///native-comments.vue").unwrap();
    let source = "\r\n<script lang=\"ts\">\r\n/* 😀\u{2028}\u{2029}\r\n 日本語\r\n*/\r\nconst value = 1;\r\n</script>";
    project.open(uri.clone(), source.into(), 7, "vue".into());
    let (query, _) = project.begin_query(&uri).unwrap();
    let retained = query.snapshot().clone();
    assert_eq!(retained.uri(), &uri);
    assert_eq!(retained.version(), 7);
    assert_eq!(retained.source(), source);
    let ready = block_on(query_sfc_comments(query, options())).unwrap();
    assert_eq!(ready.publish(|value| value), Ok(Ok(vec![expected(2, 3)])));
    // Only the actual SFC producer grants its private view, never the host key.
    assert_eq!(
        retained.native_file(&documents),
        Err(SnapshotRefusal::NativeFileUnavailable)
    );
}

#[test]
fn utf16_incremental_change_refuses_old_publication_and_refreshes_native_ranges() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = Url::parse("file:///incremental-native-comments.vue").unwrap();
    let source = "<script>\n/* 😀\n note\n*/\nconst value = 1;\n</script>";
    project.open(uri.clone(), source.into(), 1, "vue".into());
    let (query, cancel) = project.begin_query(&uri).unwrap();
    let original = query.snapshot().clone();
    let old_ready = block_on(query_sfc_comments(query, options())).unwrap();
    let change = TextDocumentContentChangeEvent {
        range: Some(Range::new(Position::new(1, 3), Position::new(1, 5))),
        range_length: None,
        text: "ok\n extra".into(),
    };
    assert!(project.apply_changes(&uri, vec![change], 2));
    assert!(cancel.is_aborted());
    let mut stale_published = false;
    assert_eq!(
        old_ready.publish(|_| stale_published = true),
        Err(SnapshotRefusal::Superseded)
    );
    assert!(!stale_published);
    let (fresh, _) = project.begin_query(&uri).unwrap();
    assert_ne!(fresh.snapshot().revision(), original.revision());
    assert_eq!(fresh.snapshot().version(), 2);
    assert_eq!(original.source(), source);
    assert_eq!(
        fresh.snapshot().source(),
        "<script>\n/* ok\n extra\n note\n*/\nconst value = 1;\n</script>"
    );
    let ready = block_on(query_sfc_comments(fresh, options())).unwrap();
    assert_eq!(ready.publish(|value| value), Ok(Ok(vec![expected(1, 3)])));
}

#[test]
fn actual_host_write_during_query_yield_has_no_guard_and_cancels_old_work() {
    let documents = DocumentStore::new();
    let project = SourceQueryProject::new(&documents);
    let uri = Url::parse("file:///yielding-native-comments.vue").unwrap();
    project.open(
        uri.clone(),
        "<script>/*\n old\n*/ const value = 1;</script>".into(),
        1,
        "vue".into(),
    );
    let (query, cancel) = project.begin_query(&uri).unwrap();
    let mut running = Box::pin(query_sfc_comments(query, options()));
    let mut context = Context::from_waker(noop_waker_ref());
    assert!(running.as_mut().poll(&mut context).is_pending());
    assert!(project.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "<script>const fresh = 2;</script>".into(),
        }],
        2
    ));
    assert!(cancel.is_aborted());
    assert!(matches!(
        running.as_mut().poll(&mut context),
        Poll::Ready(Err(SnapshotRefusal::Cancelled))
    ));
}
