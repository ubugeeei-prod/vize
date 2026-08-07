//! Deterministic Corsa checker sizing (#3905).

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
    std::env::var("VIZE_CHECKERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&count| count >= 1)
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    /// The default is pinned, not machine-derived — the whole point of #3905.
    #[test]
    fn default_is_one_checker_env_overrides() {
        // Serialized by cargo's per-process test env: mutate and restore.
        unsafe { std::env::remove_var("VIZE_CHECKERS") };
        assert_eq!(super::checker_count(), 1);
        unsafe { std::env::set_var("VIZE_CHECKERS", "6") };
        assert_eq!(super::checker_count(), 6);
        unsafe { std::env::set_var("VIZE_CHECKERS", "0") };
        assert_eq!(super::checker_count(), 1);
        unsafe { std::env::set_var("VIZE_CHECKERS", "wide") };
        assert_eq!(super::checker_count(), 1);
        unsafe { std::env::remove_var("VIZE_CHECKERS") };
    }
}
