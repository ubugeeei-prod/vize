//! Pinned production-checker admission for zero-argument slot constructors.
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
    pub(crate) mod fix_history_zero_argument;
}
use support::fix_history_runner::WorkspaceCapturePolicy;

#[test]
fn zero_argument_slot_constructors_preserve_mapper_and_authored_diagnostics() {
    support::fix_history_zero_argument::check(WorkspaceCapturePolicy { record: false });
}
