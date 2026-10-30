//! Shared native admission; the caller selects its actual evidence owner.

use super::fix_history_runner::{WorkspaceCapturePolicy, check_pack_with_capture_policy};

pub(crate) fn check(capture_policy: WorkspaceCapturePolicy) {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return;
    }
    check_pack_with_capture_policy(
        "zero-argument-slot-constructor-11013",
        "zero_argument_slot_constructors_preserve_mapper_and_authored_diagnostics",
        "a95d0aad73c8403767a507f8f0768dd0ffb265c7",
        Some(11013),
        &[
            "zero-argument-valid",
            "generic-default-zero-valid",
            "slot-string-invalid",
        ],
        capture_policy,
    );
}
