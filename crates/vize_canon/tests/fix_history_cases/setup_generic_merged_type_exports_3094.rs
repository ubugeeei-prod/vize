use crate::support::fix_history_runner::check_pack;

#[test]
fn setup_generic_and_merged_type_exports_stay_legal() {
    check_pack(
        "setup-generic-merged-type-exports-3094",
        "fix_history_cases::setup_generic_merged_type_exports_3094::setup_generic_and_merged_type_exports_stay_legal",
        "5067f25786f316af91c33beb7ef08d6f4419378e",
        Some(3094),
        &[
            "private-setup-props-public-export",
            "private-setup-props-invalid-literal",
            "generic-setup-type-export",
            "generic-setup-type-export-invalid-value",
            "merged-interface-exports",
            "merged-interface-exports-missing-member",
        ],
    );
}
