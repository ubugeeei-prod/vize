use crate::support::fix_history_runner::check_pack;

#[test]
fn every_sfc_generic_used_by_props_or_emits_is_retained() {
    check_pack(
        "every-unused-sfc-generic-2922",
        "fix_history_cases::every_unused_sfc_generic_2922::every_sfc_generic_used_by_props_or_emits_is_retained",
        "9aad33772161de7133ef2ebd628e5d21f60b43ac",
        Some(2922),
        &[
            "emit-only-generic",
            "emit-only-generic-payload-error",
            "split-props-and-emits-generics",
            "split-emits-generic-payload-error",
        ],
    );
}
