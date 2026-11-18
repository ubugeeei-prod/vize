use crate::support::fix_history_runner::check_pack;

#[test]
fn setup_scoped_props_prefixed_v_else_and_native_removed_tsconfig_options() {
    check_pack(
        "setup-deferred-props-vif-guard-tsconfig-1729",
        "fix_history_cases::setup_deferred_props_vif_guard_tsconfig_1729::setup_scoped_props_prefixed_v_else_and_native_removed_tsconfig_options",
        "fe056f2572172c469f06181bfe694878217ffc4a",
        Some(1729),
        &[
            "define-props-typeof-setup-binding",
            "define-props-typeof-setup-binding-wrong-literal",
            "deferred-props-with-hoisted-props-interface",
            "deferred-props-with-hoisted-props-wrong-literal",
            "define-props-non-hoisted-type-ref",
            "define-props-non-hoisted-type-ref-wrong-member",
            "prefixed-v-else-keeps-negated-guard",
            "prefixed-v-else-reads-other-variant-member",
            "native-removed-tsconfig-options-normalized",
            "native-removed-tsconfig-options-still-checks",
        ],
    );
}
