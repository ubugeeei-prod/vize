use crate::support::fix_history_runner::{check_pack, native_fixture_runtime_disabled};

#[test]
fn declared_any_slot_providers_preserve_authored_errors_and_intrinsic_controls() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "declared-any-slot-provider-11027",
        "fix_history_cases::declared_any_slot_provider_11027::declared_any_slot_providers_preserve_authored_errors_and_intrinsic_controls",
        "be8aa8108652c10b7eb02d963e8862fcae6b542c",
        Some(11027),
        &[
            "known",
            "declared-any",
            "error-any",
            "unknown",
            "unresolved-tag",
            "repaired",
            "unicode-crlf-any",
        ],
    );
}
