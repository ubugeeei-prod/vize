//! Deterministic Corsa checker sizing (#3905).

use crate::batch::Diagnostic;

/// Checker workers per Corsa process. Corsa's parallel checker partitions the
/// program and inference diverges across partitions: measured on the bare
/// npmx.dev fixture against tsc 6.0.3 (315 diagnostics), worker counts of
/// 4/8/14 each produce a DIFFERENT set and drop 15–18 oracle diagnostics,
/// while one worker is count-exact (#3905). Machine-derived counts also made
/// `vize check` report different errors on different machines for the same
/// commit. Pinned to one until the upstream divergence is fixed;
/// `VIZE_CHECKERS` opts back into width where throughput matters more than
/// oracle fidelity.
pub(in crate::batch::executor) fn checker_count() -> usize {
    parse_checker_count(std::env::var("VIZE_CHECKERS").ok().as_deref())
}

/// Checker workers for a program that stays unsharded. Sharding is the only
/// other parallelism, and an interconnected project collapses into a single
/// shard, so one pinned checker made its check single-threaded (#11673). Such
/// a program gets the measured width below, capped by the machine; an
/// explicit `--checkers` or `VIZE_CHECKERS` still wins. The checker-width
/// identity gate (1 vs N) guards #3905's divergence.
pub(in crate::batch::executor) fn unsharded_checker_count(
    explicit: Option<usize>,
    concurrent_programs: usize,
) -> usize {
    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    resolve_unsharded_checker_count(
        explicit,
        std::env::var("VIZE_CHECKERS").ok().as_deref(),
        threads,
        concurrent_programs,
    )
}

/// Cap the number of default-width programs that run at once so their checker
/// pools fit inside one machine budget. An explicit width is an opt-in to the
/// caller's own sizing and keeps every program eligible to run.
pub(in crate::batch::executor) fn program_concurrency_limit(
    programs: usize,
    explicit: Option<usize>,
) -> usize {
    let explicit = explicit
        .filter(|count| *count > 0)
        .or_else(|| explicit_checker_count(std::env::var("VIZE_CHECKERS").ok().as_deref()));
    if explicit.is_some() {
        return programs.max(1);
    }
    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    resolve_program_concurrency_limit(programs, threads)
}

fn resolve_program_concurrency_limit(programs: usize, threads: usize) -> usize {
    programs
        .max(1)
        .min((threads.max(1) / UNSHARDED_CHECKER_WIDTH).max(1))
}

/// Pure half of [`unsharded_checker_count`].
fn resolve_unsharded_checker_count(
    explicit: Option<usize>,
    env: Option<&str>,
    threads: usize,
    concurrent_programs: usize,
) -> usize {
    explicit
        .or_else(|| explicit_checker_count(env))
        .unwrap_or_else(|| {
            let shared_width = threads.max(1) / concurrent_programs.max(1);
            UNSHARDED_CHECKER_WIDTH.min(shared_width.max(1))
        })
}

/// The knee of a 1/2/4/8/16/32 sweep at 2.5k/10k/20k SFCs (#11673): doubling
/// to 8 still cuts wall time about 28% for at most 10% more CPU and 6% more
/// RSS, while 16 costs up to 23% more CPU and 13% more RSS for another 15%.
/// Diagnostics were identical at every width.
const UNSHARDED_CHECKER_WIDTH: usize = 8;

fn explicit_checker_count(value: Option<&str>) -> Option<usize> {
    value
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&count| count >= 1)
}

/// Pure half of [`checker_count`], so the pinning rules are testable without
/// mutating the process environment shared by every other test.
fn parse_checker_count(value: Option<&str>) -> usize {
    value
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&count| count >= 1)
        .unwrap_or(1)
}

/// Whether the run failed because the runtime does not know `--checkers`: an
/// older Corsa rejects the whole invocation with TS5023. Retrying without the
/// option would silently check the project at Corsa's default checker width,
/// whose diagnostic set differs from the pinned one-checker oracle (#3905),
/// the exact machine-dependent drift this pin removes, so callers must fail.
pub(super) fn rejects_checkers_flag(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == Some(5023) && diagnostic.message.contains("checkers"))
}

#[cfg(test)]
mod tests {
    use super::{
        explicit_checker_count, parse_checker_count, resolve_program_concurrency_limit,
        resolve_unsharded_checker_count,
    };

    /// The default is pinned, not machine-derived — the whole point of #3905.
    #[test]
    fn default_is_one_checker() {
        assert_eq!(parse_checker_count(None), 1);
    }

    #[test]
    fn unsharded_programs_take_the_default_width_unless_a_width_is_explicit() {
        assert_eq!(
            resolve_unsharded_checker_count(None, None, 32, 1),
            super::UNSHARDED_CHECKER_WIDTH
        );
        // Never more workers than the machine has.
        assert_eq!(resolve_unsharded_checker_count(None, None, 2, 1), 2);
        assert_eq!(resolve_unsharded_checker_count(None, None, 0, 1), 1);
        // Concurrent programs share the machine budget without exceeding the
        // measured single-program knee.
        assert_eq!(resolve_unsharded_checker_count(None, None, 32, 2), 8);
        assert_eq!(resolve_unsharded_checker_count(None, None, 32, 8), 4);
        assert_eq!(resolve_unsharded_checker_count(None, None, 4, 8), 1);
        // `--checkers` and `VIZE_CHECKERS` both pin it, `--checkers` first.
        assert_eq!(
            resolve_unsharded_checker_count(Some(1), Some("8"), 32, 8),
            1
        );
        assert_eq!(resolve_unsharded_checker_count(None, Some("1"), 32, 8), 1);
        assert_eq!(
            resolve_unsharded_checker_count(None, Some("wide"), 32, 1),
            super::UNSHARDED_CHECKER_WIDTH
        );
    }

    #[test]
    fn default_width_programs_fit_inside_one_machine_budget() {
        assert_eq!(resolve_program_concurrency_limit(8, 32), 4);
        assert_eq!(resolve_program_concurrency_limit(2, 32), 2);
        assert_eq!(resolve_program_concurrency_limit(8, 4), 1);
        assert_eq!(resolve_program_concurrency_limit(0, 0), 1);
    }

    #[test]
    fn only_a_valid_override_is_explicit() {
        assert_eq!(explicit_checker_count(None), None);
        assert_eq!(explicit_checker_count(Some("0")), None);
        assert_eq!(explicit_checker_count(Some("wide")), None);
        assert_eq!(explicit_checker_count(Some("3")), Some(3));
    }

    #[test]
    fn env_opts_back_into_width() {
        assert_eq!(parse_checker_count(Some("6")), 6);
        assert_eq!(parse_checker_count(Some("1")), 1);
    }

    /// A zero or unparsable value must not disable checking or panic; it falls
    /// back to the pinned default.
    #[test]
    fn rejects_zero_and_unparsable_values() {
        assert_eq!(parse_checker_count(Some("0")), 1);
        assert_eq!(parse_checker_count(Some("wide")), 1);
        assert_eq!(parse_checker_count(Some("")), 1);
        assert_eq!(parse_checker_count(Some("-4")), 1);
    }
}
