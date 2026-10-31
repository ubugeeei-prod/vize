use crate::support::fix_history_runner::{check_pack, native_fixture_runtime_disabled};

pub(crate) fn check() {
    if native_fixture_runtime_disabled() {
        return;
    }
    check_pack(
        "unknown-component-slot-payload-11027",
        "unknown_component_slots_preserve_authored_type_diagnostics",
        "99ee7d1c4b99f9121f70a5f60d5f9aa81c596d47",
        Some(11027),
        &[
            "valid",
            "unknown",
            "unknown-union",
            "repaired",
            "unresolved-tag",
            "slot-any",
            "slot-unknown",
            "slot-never",
            "slot-string",
            "unicode-crlf-unknown",
        ],
    );
}
