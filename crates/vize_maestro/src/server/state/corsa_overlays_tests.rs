//! Correctness tests for the incremental Corsa overlay cache.
//!
//! The scaling half lives in [`super::corsa_overlays_perf_tests`], which
//! reuses the fixtures below.

use std::path::PathBuf;
use std::sync::Arc;

use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};
use vize_s0::cstr;

use super::ServerState;

pub(super) fn uri(name: &str) -> Url {
    Url::parse(&cstr!("file:///project/{name}")).expect("uri")
}

pub(super) fn path(name: &str) -> PathBuf {
    uri(name).to_file_path().expect("file path")
}

pub(super) fn open(state: &ServerState, name: &str, text: &str) {
    state
        .documents
        .open(uri(name), text.to_string(), 1, "vue".to_string());
}

/// Replace the whole buffer, the way an editor reports a non-incremental edit.
pub(super) fn rewrite(state: &ServerState, name: &str, text: &str, version: i32) {
    state.documents.apply_changes(
        &uri(name),
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: text.to_string(),
        }],
        version,
    );
}

/// The overlay set as compared by tests: order is an artifact of `DashMap`
/// iteration, so sort before asserting full equality.
pub(super) fn overlays(state: &ServerState) -> Vec<(PathBuf, String)> {
    let mut overlays = state
        .corsa_overlays()
        .into_iter()
        .map(|(path, text)| (path, text.to_string()))
        .collect::<Vec<_>>();
    overlays.sort();
    overlays
}

/// The stale-overlay guard: a wrong overlay makes Corsa type-check text the
/// user never wrote, so the whole set is asserted after each step of a
/// realistic editing session rather than spot-checked.
#[test]
fn overlay_set_tracks_opens_edits_and_closes() {
    let state = ServerState::new();

    open(&state, "A.vue", "<template>A1</template>");
    assert_eq!(
        overlays(&state),
        vec![(path("A.vue"), "<template>A1</template>".to_string())]
    );

    open(&state, "B.vue", "<template>B1</template>");
    assert_eq!(
        overlays(&state),
        vec![
            (path("A.vue"), "<template>A1</template>".to_string()),
            (path("B.vue"), "<template>B1</template>".to_string()),
        ]
    );

    rewrite(&state, "A.vue", "<template>A2</template>", 2);
    assert_eq!(
        overlays(&state),
        vec![
            (path("A.vue"), "<template>A2</template>".to_string()),
            (path("B.vue"), "<template>B1</template>".to_string()),
        ]
    );

    state.documents.close(&uri("B.vue"));
    assert_eq!(
        overlays(&state),
        vec![(path("A.vue"), "<template>A2</template>".to_string())]
    );

    rewrite(&state, "A.vue", "<template>A3</template>", 3);
    assert_eq!(
        overlays(&state),
        vec![(path("A.vue"), "<template>A3</template>".to_string())]
    );
}

/// A reopen restarts the client's `version` at 1, so a version-keyed cache
/// would serve the closed document's text. Revisions are monotonic and never
/// reused, so this reads the new content.
#[test]
fn reopening_a_document_at_the_same_version_is_not_served_from_cache() {
    let state = ServerState::new();

    open(&state, "A.vue", "<template>first</template>");
    assert_eq!(
        overlays(&state),
        vec![(path("A.vue"), "<template>first</template>".to_string())]
    );

    state.documents.close(&uri("A.vue"));
    open(&state, "A.vue", "<template>second</template>");
    assert_eq!(
        overlays(&state),
        vec![(path("A.vue"), "<template>second</template>".to_string())]
    );
}

/// An in-place edit keeps the same `Document`, so the cache has to notice the
/// rope changed underneath an entry it already holds.
#[test]
fn incremental_edits_are_reflected_in_the_overlay_set() {
    let state = ServerState::new();
    open(&state, "A.vue", "<template>ab</template>");
    assert_eq!(
        overlays(&state),
        vec![(path("A.vue"), "<template>ab</template>".to_string())]
    );

    state.documents.apply_changes(
        &uri("A.vue"),
        vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: 0,
                    character: 11,
                },
                end: Position {
                    line: 0,
                    character: 12,
                },
            }),
            range_length: None,
            text: "X".to_string(),
        }],
        2,
    );
    assert_eq!(
        overlays(&state),
        vec![(path("A.vue"), "<template>aX</template>".to_string())]
    );
}

/// A rename moves text to a new path; the overlay must follow and the old path
/// must not linger.
#[test]
fn renaming_a_document_moves_its_overlay() {
    let state = ServerState::new();
    open(&state, "Old.vue", "<template>same</template>");
    assert_eq!(
        overlays(&state),
        vec![(path("Old.vue"), "<template>same</template>".to_string())]
    );

    assert!(state.rename_document(&uri("Old.vue"), uri("New.vue")));
    assert_eq!(state.corsa_overlay_entries(), 0);
    assert_eq!(
        overlays(&state),
        vec![(path("New.vue"), "<template>same</template>".to_string())]
    );
}

/// Invalidation may only cost work, never correctness: the set it rebuilds is
/// the same one it dropped.
#[test]
fn invalidation_rebuilds_an_identical_overlay_set() {
    let state = ServerState::new();
    open(&state, "A.vue", "<template>A</template>");
    open(&state, "B.vue", "<template>B</template>");
    let before = overlays(&state);

    state.invalidate_corsa_overlays();
    let after = overlays(&state);

    assert_eq!(before, after);
    assert_eq!(
        after,
        vec![
            (path("A.vue"), "<template>A</template>".to_string()),
            (path("B.vue"), "<template>B</template>".to_string()),
        ]
    );
}

#[test]
fn closing_a_document_releases_its_cached_overlay_without_another_pass() {
    let state = ServerState::new();
    open(&state, "A.vue", &"large buffer".repeat(4_096));
    let _ = state.corsa_overlays();
    assert_eq!(state.corsa_overlay_entries(), 1);

    state.close_document(&uri("A.vue"));

    assert_eq!(state.corsa_overlay_entries(), 0);
    assert!(state.documents.is_empty());
}

#[test]
fn concurrent_edits_and_snapshots_finish_with_the_latest_text() {
    let state = Arc::new(ServerState::new());
    open(&state, "A.vue", "version 1");
    let reader = Arc::clone(&state);
    let snapshots = std::thread::spawn(move || {
        for _ in 0..128 {
            let _ = reader.corsa_overlays();
        }
    });

    for version in 2..=128 {
        rewrite(&state, "A.vue", &cstr!("version {version}"), version);
    }
    snapshots.join().expect("snapshot thread");

    assert_eq!(
        overlays(&state),
        vec![(path("A.vue"), "version 128".into())]
    );
}
