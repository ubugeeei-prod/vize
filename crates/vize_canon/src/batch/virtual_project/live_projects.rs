//! Test-only count of live projects, charged to the thread that built them
//! wherever they are dropped.
#![expect(
    clippy::disallowed_types,
    reason = "a worker can drop the last project"
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
struct Counts {
    live: AtomicUsize,
    peak: AtomicUsize,
}

thread_local! {
    static COUNTS: Arc<Counts> = Arc::default();
}

pub(super) struct LiveProject(Arc<Counts>);

impl LiveProject {
    pub(super) fn new() -> Self {
        COUNTS.with(|counts| {
            let live = counts.live.fetch_add(1, Ordering::SeqCst) + 1;
            counts.peak.fetch_max(live, Ordering::SeqCst);
            Self(Arc::clone(counts))
        })
    }
}

impl Drop for LiveProject {
    fn drop(&mut self) {
        self.0.live.fetch_sub(1, Ordering::SeqCst);
    }
}

pub(crate) fn live_projects() -> usize {
    COUNTS.with(|counts| counts.live.load(Ordering::SeqCst))
}

/// The most projects alive at once since the last call.
pub(crate) fn take_peak_projects() -> usize {
    COUNTS.with(|counts| {
        let live = counts.live.load(Ordering::SeqCst);
        counts.peak.swap(live, Ordering::SeqCst)
    })
}
