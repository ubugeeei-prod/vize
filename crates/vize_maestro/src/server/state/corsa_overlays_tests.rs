//! Correctness and scaling tests for the incremental Corsa overlay cache.

use std::path::PathBuf;
use std::sync::Arc;

use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

use super::ServerState;

fn uri(name: &str) -> Url {
    Url::parse(&format!("file:///project/{name}")).expect("uri")
}

fn path(name: &str) -> PathBuf {
    uri(name).to_file_path().expect("file path")
}

fn open(state: &ServerState, name: &str, text: &str) {
    state
        .documents
        .open(uri(name), text.to_string(), 1, "vue".to_string());
}

/// Replace the whole buffer, the way an editor reports a non-incremental edit.
fn rewrite(state: &ServerState, name: &str, text: &str, version: i32) {
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
fn overlays(state: &ServerState) -> Vec<(PathBuf, String)> {
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
        rewrite(&state, "A.vue", &format!("version {version}"), version);
    }
    snapshots.join().expect("snapshot thread");

    assert_eq!(
        overlays(&state),
        vec![(path("A.vue"), "version 128".into())]
    );
}

/// The point of the cache: a pass costs the documents that changed, not the
/// documents that are open. Counting materializations rather than timing keeps
/// the assertion exact and independent of the machine running it.
#[test]
fn a_pass_materializes_only_the_documents_that_changed() {
    let state = ServerState::new();
    let document = "<template>{{ value }}</template>\n".repeat(512);
    for index in 0..40 {
        open(&state, &format!("C{index}.vue"), &document);
    }

    // Cold pass reads every open document exactly once.
    let _ = state.corsa_overlays();
    assert_eq!(state.corsa_overlay_materializations(), 40);

    // A pass with nothing changed reads none of them.
    let _ = state.corsa_overlays();
    assert_eq!(state.corsa_overlay_materializations(), 40);

    // Ten keystrokes on one file read one document each, regardless of the 40
    // that are open.
    for keystroke in 0..10 {
        rewrite(
            &state,
            "C7.vue",
            &format!("{document}<!-- {keystroke} -->"),
            keystroke + 2,
        );
        let _ = state.corsa_overlays();
    }
    assert_eq!(state.corsa_overlay_materializations(), 50);

    // Opening one more document reads only that one.
    open(&state, "C40.vue", &document);
    let _ = state.corsa_overlays();
    assert_eq!(state.corsa_overlay_materializations(), 51);

    // Closing one reads nothing.
    state.close_document(&uri("C40.vue"));
    let _ = state.corsa_overlays();
    assert_eq!(state.corsa_overlay_materializations(), 51);
}

/// The wall-clock companion to the deterministic counter test above, and the
/// source of the numbers in the PR body. Run manually on an idle host: scheduler
/// contention makes elapsed-time assertions unsuitable as a CI correctness
/// gate, while `a_pass_materializes_only_the_documents_that_changed` catches
/// the same full-rebuild regression exactly.
#[test]
#[ignore = "wall-clock benchmark; run manually on an idle host"]
fn a_warm_pass_is_far_cheaper_than_a_cold_one() {
    const DOCUMENTS: usize = 60;
    const PASSES: u32 = 64;

    let state = ServerState::new();
    let document = "<template>{{ value }}</template>\n".repeat(512);
    for index in 0..DOCUMENTS {
        open(&state, &format!("D{index}.vue"), &document);
    }

    // Prime the cache, then interleave warm/cold samples so host load affects
    // both populations equally. Edits and invalidation are setup, not overlay
    // snapshot work, and therefore deliberately stay outside the timers.
    let _ = state.corsa_overlays();
    let mut warm = std::time::Duration::ZERO;
    let mut cold = std::time::Duration::ZERO;
    for keystroke in 0..PASSES {
        rewrite(
            &state,
            "D3.vue",
            &format!("{document}<!-- {keystroke} -->"),
            keystroke as i32 + 2,
        );

        let started = std::time::Instant::now();
        let _ = state.corsa_overlays();
        warm += started.elapsed();

        state.invalidate_corsa_overlays();
        let started = std::time::Instant::now();
        let _ = state.corsa_overlays();
        cold += started.elapsed();
    }
    let warm = warm / PASSES;
    let cold = cold / PASSES;

    println!(
        "corsa overlay pass over {DOCUMENTS} documents: cold {cold:?}, warm (1 edited) {warm:?}"
    );
    assert!(
        warm * 4 < cold,
        "a warm pass should cost a fraction of a full rebuild, got warm {warm:?} vs cold {cold:?}"
    );
}
