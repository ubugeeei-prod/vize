use crate::support::fix_history_runner::check_pack;

#[test]
fn strict_template_alignment_bundle_preserves_checked_diagnostics() {
    check_pack(
        "strict-template-diagnostics-alignment",
        "fix_history_cases::strict_template_diagnostics_alignment::strict_template_alignment_bundle_preserves_checked_diagnostics",
        "1acdeec5fc51a7f1d270fef08c4c65467e7f0821",
        None,
        &[
            "external-esm-augmentation-applies",
            "external-esm-augmented-member-misuse-invalid",
            "v-if-inside-v-for-template-sees-alias",
            "same-element-v-if-before-v-for-alias-invalid",
            "component-event-modifier-pair",
            "spread-listener-prop-stays-checked-invalid",
            "generic-slot-authored-default",
            "generic-slot-authored-default-mismatch-invalid",
            "explicit-ts-import-adjacent-declaration",
            "explicit-ts-import-adjacent-declaration-invalid",
        ],
    );
}
