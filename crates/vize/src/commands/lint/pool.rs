//! The lint worker pool, sized to the inputs a run found.
//!
//! Rayon's global pool starts one worker per CPU the first time anything asks
//! for it, and starting them is serial work on the main thread. A run over a
//! few files then waits for every worker to start, and the idle ones still
//! spin before they sleep. Discovery therefore reads the worker budget without
//! starting the pool, and the pool starts once the inputs are known, with no
//! more workers than they can keep busy.

use std::path::PathBuf;

/// Source bytes one more worker has to have before it pays for its start and
/// its idle spinning. A typical SFC is a few KiB, so this is about eight files.
const BYTES_PER_WORKER: u64 = 32 * 1024;

/// Workers the run may use: `RAYON_NUM_THREADS` when set, else one per CPU,
/// exactly as Rayon would choose without starting the pool to ask it.
pub(super) fn budget() -> usize {
    std::env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|threads| threads.parse::<usize>().ok())
        .filter(|threads| *threads > 0)
        .or_else(|| std::thread::available_parallelism().ok().map(usize::from))
        .unwrap_or(1)
}

/// Start the global pool with as many workers as `files` can occupy: never
/// more than one per file, and below the budget only when the files are small
/// enough to be counted (a `stat` each). A pool something already started
/// keeps its size.
pub(super) fn start_for(files: &[PathBuf]) {
    let budget = budget();
    let workers = if files.len() <= budget.saturating_mul(8) {
        let bytes: u64 = files
            .iter()
            .map(|file| std::fs::metadata(file).map_or(BYTES_PER_WORKER, |meta| meta.len()))
            .sum();
        workers_for(files.len(), bytes, budget)
    } else {
        budget
    };
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build_global();
}

fn workers_for(files: usize, bytes: u64, budget: usize) -> usize {
    let by_bytes = usize::try_from(bytes.div_ceil(BYTES_PER_WORKER)).unwrap_or(usize::MAX);
    budget.min(files).min(by_bytes).max(1)
}

#[cfg(test)]
mod tests {
    use super::workers_for;

    #[test]
    fn workers_never_exceed_files_budget_or_byte_share() {
        assert_eq!(workers_for(1, 400_000, 32), 1);
        assert_eq!(workers_for(40, 160 * 1024, 32), 5);
        assert_eq!(workers_for(40, 0, 32), 1);
        assert_eq!(workers_for(16, 16 * 1024 * 1024, 32), 16);
        assert_eq!(workers_for(900, 4 * 1024 * 1024, 32), 32);
        assert_eq!(workers_for(900, 4 * 1024 * 1024, 4), 4);
    }
}
