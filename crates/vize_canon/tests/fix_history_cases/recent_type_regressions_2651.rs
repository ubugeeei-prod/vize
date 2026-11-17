use crate::support::fix_history_runner::check_pack;

#[test]
fn recent_generic_ref_interface_slot_and_expose_regressions() {
    check_pack(
        "recent-type-regressions-2651",
        "fix_history_cases::recent_type_regressions_2651::recent_generic_ref_interface_slot_and_expose_regressions",
        "984cc2c1c1d0f4e120fc2a968aef04e3c0ee58f0",
        Some(2651),
        &[
            "issue-2645-generic-sfc-props-in-tsx",
            "issue-2645-generic-sfc-missing-required-prop-in-tsx",
            "issue-2646-generic-template-ref-not-widened",
            "issue-2646-generic-template-ref-keeps-undefined",
            "issue-2647-setup-type-import-and-local-alias",
            "issue-2647-local-alias-still-checked",
            "issue-2648-imported-interface-props-in-template",
            "issue-2648-imported-interface-prop-wrong-member",
            "issue-2649-v-slot-scope-spread",
            "issue-2649-number-spread-rejected",
            "issue-2650-exposed-instance-cast",
            "issue-2650-exposed-instance-missing-member",
        ],
    );
}
