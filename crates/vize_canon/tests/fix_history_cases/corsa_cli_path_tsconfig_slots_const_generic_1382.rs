use crate::support::fix_history_runner::check_pack;

#[test]
fn annotated_slots_const_generics_and_inlined_tsconfig_chains() {
    check_pack(
        "corsa-cli-path-tsconfig-slots-const-generic-1382",
        "fix_history_cases::corsa_cli_path_tsconfig_slots_const_generic_1382::annotated_slots_const_generics_and_inlined_tsconfig_chains",
        "1fdbb7ad0c88d27a55d4fa9056b2140916d2ac17",
        Some(1382),
        &[
            "annotated-slot-props-keep-user-annotation",
            "annotated-slot-props-mismatch-child-slot-type",
            "const-generic-component",
            "const-generic-component-wrong-member",
            "extends-array-chain-inlined",
            "extends-array-chain-inherits-strict",
            "custom-type-roots-reanchored",
            "custom-type-roots-global-wrong-type",
            "custom-type-roots-missing-entry-fileless-diagnostic",
        ],
    );
}
