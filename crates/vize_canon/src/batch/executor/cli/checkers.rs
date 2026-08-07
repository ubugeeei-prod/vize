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
pub(super) fn checker_count() -> usize {
    parse_checker_count(std::env::var("VIZE_CHECKERS").ok().as_deref())
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
    use super::parse_checker_count;

    /// The default is pinned, not machine-derived — the whole point of #3905.
    #[test]
    fn default_is_one_checker() {
        assert_eq!(parse_checker_count(None), 1);
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
