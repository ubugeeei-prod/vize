//! The real native BatchTypeChecker owns all authored alias and negative controls.
use super::check_pack;

#[test]
fn authored_propertykey_alias_preserves_batch_diagnostics() {
    check_pack(
        "aria-propertykey-capture-11153",
        "aria_propertykey_capture_11153::authored_propertykey_alias_preserves_batch_diagnostics",
        "aed91c7626f9daf11a07b9eb32a0b7ff27ec9038",
        Some(11153),
        &[
            "reported-alias",
            "reported-control",
            "alias-valid",
            "alias-invalid",
            "control-valid",
            "control-invalid",
        ],
    );
}
