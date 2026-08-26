//! Scaling tests for the incremental Corsa overlay cache.
//!
//! Split from [`super::corsa_overlays_tests`] to stay inside the
//! per-file line budget; the fixtures are shared from there.

use vize_s0::cstr;

use super::ServerState;
use super::corsa_overlays_tests::{open, rewrite, uri};

/// The point of the cache: a pass costs the documents that changed, not the
/// documents that are open. Counting materializations rather than timing keeps
/// the assertion exact and independent of the machine running it.
#[test]
fn a_pass_materializes_only_the_documents_that_changed() {
    let state = ServerState::new();
    let document = "<template>{{ value }}</template>\n".repeat(512);
    for index in 0..40 {
        open(&state, &cstr!("C{index}.vue"), &document);
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
            &cstr!("{document}<!-- {keystroke} -->"),
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

/// Average cost of one overlay pass over `documents` open buffers of
/// `bytes` each: `(cold, warm)`, where cold rebuilds the whole set the way the
/// pre-#3442 code did and warm follows a single edit.
///
/// Samples alternate so host load lands on both populations equally, and the
/// edits and invalidation that set each sample up stay outside the timers.
fn overlay_pass_cost(
    documents: usize,
    bytes: usize,
    passes: u32,
) -> (std::time::Duration, std::time::Duration) {
    let state = ServerState::new();
    let document = "<template>{{ value }}</template>\n".repeat(bytes / 33);
    for index in 0..documents {
        open(&state, &cstr!("D{index}.vue"), &document);
    }

    let _ = state.corsa_overlays();
    let mut warm = std::time::Duration::ZERO;
    let mut cold = std::time::Duration::ZERO;
    for keystroke in 0..passes {
        rewrite(
            &state,
            "D3.vue",
            &cstr!("{document}<!-- {keystroke} -->"),
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
    (cold / passes, warm / passes)
}

/// The wall-clock companion to the deterministic counter test above, and the
/// source of the numbers in the PR body. Run manually on an idle host: scheduler
/// contention makes elapsed-time assertions unsuitable as a CI correctness
/// gate, while `a_pass_materializes_only_the_documents_that_changed` catches
/// the same full-rebuild regression exactly.
///
/// Sweeping the buffer size at a fixed document count is what isolates the
/// claim. A cold pass copies every open buffer, so its cost tracks
/// `documents x bytes`; a warm pass copies only the edited one, so growing the
/// buffers must leave it flat. Any residual growth in the warm column is the
/// per-document bookkeeping (path clone, hash lookup) that survives caching.
#[test]
#[ignore = "wall-clock benchmark; run manually on an idle host"]
fn a_warm_pass_does_not_scale_with_open_buffer_size() {
    const DOCUMENTS: usize = 60;
    const PASSES: u32 = 64;

    let mut measurements = Vec::new();
    for bytes in [8 * 1024, 32 * 1024, 128 * 1024] {
        let (cold, warm) = overlay_pass_cost(DOCUMENTS, bytes, PASSES);
        println!(
            "{DOCUMENTS} open buffers of {}KiB: cold {cold:?}, warm (1 edited) {warm:?}",
            bytes / 1024
        );
        measurements.push((bytes, cold, warm));
    }

    let (_, smallest_cold, smallest_warm) = measurements[0];
    let (_, largest_cold, largest_warm) = measurements[measurements.len() - 1];

    // 16x the bytes per buffer. The cold pass has to copy all of them.
    assert!(
        largest_cold > smallest_cold * 4,
        "a cold pass must track total open bytes, got {smallest_cold:?} -> {largest_cold:?}"
    );
    // The warm pass copies one buffer regardless, so it must not follow.
    assert!(
        largest_warm < smallest_warm * 4,
        "a warm pass must not track total open bytes, got {smallest_warm:?} -> {largest_warm:?}"
    );
    assert!(
        largest_warm * 4 < largest_cold,
        "a warm pass should cost a fraction of a full rebuild, \
         got warm {largest_warm:?} vs cold {largest_cold:?}"
    );
}
