use crate::support::fix_history_runner::check_pack;

#[test]
fn setup_ref_destructuring_defaults_and_narrowed_maybe_refs_unwrap() {
    check_pack(
        "setup-ref-destructuring-narrowed-unwrap-6260",
        "fix_history_cases::setup_ref_destructuring_narrowed_unwrap_6260::setup_ref_destructuring_defaults_and_narrowed_maybe_refs_unwrap",
        "215ca5706f402ed831a3bac0166e96cbc21da286",
        Some(6260),
        &[
            "destructured-loop-defaults-read-unwrapped-ref",
            "destructured-loop-defaults-narrow-parameter-errors",
            "narrowed-maybe-ref-read-through-value",
            "non-narrowing-guard-keeps-undefined",
        ],
    );
}
