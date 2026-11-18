//! The lint worker pool, sized to explicitly named inputs.
//!
//! Rayon's global pool starts one worker per CPU the first time anything asks
//! for it. Directory discovery asks while it walks, so a directory or glob run
//! starts every worker early, while the walk still has work for the main
//! thread. A run over named files (an editor, `lint-staged`) skips discovery
//! and would otherwise wait for every worker to start, then leave all but one
//! per file idle; it starts the pool here with at most one worker per file.

/// Workers the run may use: `RAYON_NUM_THREADS` when set, else one per CPU,
/// exactly as Rayon would choose without starting the pool to ask it.
fn budget() -> usize {
    std::env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|threads| threads.parse::<usize>().ok())
        .filter(|threads| *threads > 0)
        .or_else(|| std::thread::available_parallelism().ok().map(usize::from))
        .unwrap_or(1)
}

/// Start the global pool with no more workers than `files`: each file is
/// linted on one worker, so another worker would only start and sleep. A pool
/// discovery already started keeps its size.
pub(super) fn start_for(files: usize) {
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(workers_for(files, budget()))
        .build_global();
}

fn workers_for(files: usize, budget: usize) -> usize {
    budget.min(files).max(1)
}

#[cfg(test)]
mod tests {
    use super::workers_for;

    #[test]
    fn workers_never_exceed_files_or_budget() {
        assert_eq!(workers_for(1, 32), 1);
        assert_eq!(workers_for(8, 32), 8);
        assert_eq!(workers_for(0, 32), 1);
        assert_eq!(workers_for(900, 32), 32);
        assert_eq!(workers_for(900, 4), 4);
    }
}
