//! Exact batch diagnostics for history-free dependency resolution.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]
#![expect(clippy::disallowed_types, reason = "JSON fixtures use std strings")]

mod support {
    pub(crate) mod fix_history_contract;
    pub(crate) mod fix_history_normalization;
    pub(crate) mod fix_history_observation;
    pub(crate) mod fix_history_paths;
    pub(crate) mod fix_history_reference_paths;
    pub(crate) mod fix_history_router_provider;
    pub(crate) mod fix_history_runner;
}

#[test]
fn dependency_probe_histories_preserve_complete_diagnostics() {
    support::fix_history_runner::check_pack(
        "dependency-probe-history-10936",
        "dependency_probe_histories_preserve_complete_diagnostics",
        "1266cd3a014654bbdcf3378500b93447411b26d9",
        Some(10936),
        &[
            "valid-precedence-and-alias",
            "invalid-authored-model",
            "repaired-authored-model",
        ],
    );
}
