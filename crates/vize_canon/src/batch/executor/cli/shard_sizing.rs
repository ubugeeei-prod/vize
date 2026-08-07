//! How wide to shard a project's Corsa run (#3905).

/// How many concurrent Corsa programs to run on a machine of `threads` cores.
///
/// A process occupies about `checkers` cores, since that is the width of its
/// checker pool: one by default now that the count is pinned for determinism
/// (#3905), not Corsa's old four-wide default. Dividing by that width is what
/// keeps the processes covering the machine: with the pin, a divisor of 4 left
/// three quarters of a wide runner idle. The division floors so the checker
/// workers stay within the thread budget: rounding up would oversubscribe the
/// machine whenever the width does not divide the cores. Sharding only pays off
/// once there are enough Vue files to amortize each extra program's fixed
/// parse/bind cost, and the upper bound keeps that duplicated cost contained.
pub(super) fn shard_count(threads: usize, checkers: usize, vue_files: usize) -> usize {
    (threads / checkers.max(1)).min(vue_files / 64).clamp(1, 8)
}

#[cfg(test)]
mod tests {
    use super::shard_count;

    /// With the checker count pinned to one (#3905), a process occupies one
    /// core, so the shards have to cover the machine themselves.
    #[test]
    fn shards_cover_the_machine_at_one_checker_per_process() {
        // 32-vCPU runner, 1000-file corpus: the old fixed divisor of 4 (plus a
        // cap of 4) left four single-checker processes on 32 cores.
        assert_eq!(shard_count(32, 1, 1000), 8);
        assert_eq!(shard_count(8, 1, 1000), 8);
        assert_eq!(shard_count(4, 1, 1000), 4);
        assert_eq!(shard_count(1, 1, 1000), 1);
    }

    #[test]
    fn shard_count_accounts_for_wider_checker_pools() {
        // `VIZE_CHECKERS` opts back into width: four checkers per process means
        // a quarter as many processes for the same cores.
        assert_eq!(shard_count(32, 4, 1000), 8);
        assert_eq!(shard_count(8, 4, 1000), 2);
        assert_eq!(shard_count(8, 8, 1000), 1);
        // A degenerate zero must not divide by zero.
        assert_eq!(shard_count(8, 0, 1000), 8);
    }

    /// A width that does not divide the cores must not oversubscribe them.
    #[test]
    fn shard_count_stays_within_the_thread_budget() {
        for (threads, checkers) in [(32usize, 5usize), (8, 3), (6, 4), (10, 7)] {
            let shards = shard_count(threads, checkers, 1000);
            assert!(
                shards == 1 || shards * checkers <= threads,
                "{shards} shards of {checkers} checkers exceed {threads} threads"
            );
        }
        assert_eq!(shard_count(32, 5, 1000), 6);
    }

    /// Small projects stay unsharded: an extra program's parse/bind cost is not
    /// amortized until there are enough Vue files to spread.
    #[test]
    fn small_projects_stay_unsharded() {
        assert_eq!(shard_count(32, 1, 0), 1);
        assert_eq!(shard_count(32, 1, 63), 1);
        assert_eq!(shard_count(32, 1, 127), 1);
        assert_eq!(shard_count(32, 1, 128), 2);
    }
}
