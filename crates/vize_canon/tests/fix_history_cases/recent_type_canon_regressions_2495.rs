use crate::support::fix_history_runner::check_pack;

#[test]
fn interface_props_contextmenu_and_generic_specialization() {
    check_pack(
        "recent-type-canon-regressions-2495",
        "fix_history_cases::recent_type_canon_regressions_2495::interface_props_contextmenu_and_generic_specialization",
        "25ca8a2b12bccf4dd321f0e152275233501b3883",
        Some(2495),
        &[
            "imported-base-interface-template-props",
            "imported-base-interface-prop-wrong-member",
            "contextmenu-accepts-pointer-event-handler",
            "contextmenu-rejects-keyboard-event-handler",
            "generic-sfc-value-specialized-from-typescript",
            "generic-sfc-value-rejects-extra-type-argument",
            "nested-union-prop-members-stay-nested",
            "nested-union-member-is-not-a-template-binding",
        ],
    );
}
