//! The ignore-aware walk behind lint discovery.

use ignore::{DirEntry, WalkBuilder, WalkState};
use std::path::PathBuf;
use std::sync::Mutex;

/// Directory readers for one walk. Past four, n8n's 37k-entry tree walked no
/// faster and spent more kernel time on directory locks.
const MAX_WALK_THREADS: usize = 4;

/// Every walked entry `selected` accepts, in no particular order.
///
/// One lint worker walks in order on the calling thread. More workers read
/// directories concurrently on up to [`MAX_WALK_THREADS`] threads.
pub(super) fn candidates(
    walker: &mut WalkBuilder,
    selected: impl Fn(&DirEntry) -> bool + Sync,
) -> Vec<PathBuf> {
    // The budget, not the pool: starting the pool here would size it before
    // the run knows how many files it has.
    let threads = super::super::pool::budget().min(MAX_WALK_THREADS);
    if threads <= 1 {
        return walker
            .build()
            .filter_map(Result::ok)
            .filter(|entry| selected(entry))
            .map(DirEntry::into_path)
            .collect();
    }
    let found = Mutex::new(Vec::new());
    walker.threads(threads).build_parallel().run(|| {
        let selected = &selected;
        let found = &found;
        Box::new(move |entry| {
            if let Ok(entry) = entry
                && selected(&entry)
                && let Ok(mut found) = found.lock()
            {
                found.push(entry.into_path());
            }
            WalkState::Continue
        })
    });
    found.into_inner().unwrap_or_default()
}
