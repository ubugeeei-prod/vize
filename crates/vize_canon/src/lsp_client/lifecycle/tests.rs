use std::path::PathBuf;

use super::{merge_materialized_file_changes, vue_virtual_document_uris};
use corsa::api::{DocumentIdentifier, FileChangeSummary, FileChanges};
use vize_s0::cstr;

#[test]
fn merges_materialized_file_change_summaries() {
    let mut summary = FileChangeSummary::default();
    merge_materialized_file_changes(
        &mut summary,
        Some(FileChanges::Summary(FileChangeSummary {
            changed: vec![DocumentIdentifier::from("/workspace/a.ts")],
            created: vec![DocumentIdentifier::from("/workspace/b.ts")],
            deleted: Vec::new(),
        })),
    );
    merge_materialized_file_changes(
        &mut summary,
        Some(FileChanges::Summary(FileChangeSummary {
            changed: vec![DocumentIdentifier::from("/workspace/c.ts")],
            created: Vec::new(),
            deleted: vec![DocumentIdentifier::from("/workspace/d.ts")],
        })),
    );

    assert_eq!(summary.changed.len(), 2);
    assert_eq!(summary.created.len(), 1);
    assert_eq!(summary.deleted.len(), 1);
}

#[cfg(unix)]
#[test]
fn deleted_vue_sources_expand_to_deduplicated_ts_and_tsx_overlay_uris() {
    let source = PathBuf::from("/workspace/Panel #1.vue");
    assert_eq!(
        vue_virtual_document_uris(&[
            source.clone(),
            source,
            PathBuf::from("/workspace/ordinary.ts"),
        ]),
        vec![
            cstr!("file:///workspace/Panel%20%231.vue.ts"),
            cstr!("file:///workspace/Panel%20%231.vue.tsx"),
        ]
    );
}
