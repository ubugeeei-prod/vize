use crate::support::fix_history_runner::check_pack;

#[test]
fn setup_scoped_value_dependent_type_exports_are_public() {
    check_pack(
        "setup-scoped-type-exports-3093",
        "fix_history_cases::setup_scoped_type_exports_3093::setup_scoped_value_dependent_type_exports_are_public",
        "71e38ec49f6931f5341d433bea8709d767daba87",
        Some(3093),
        &[
            "value-dependent-type-exports",
            "value-dependent-type-exports-reject-invalid-literals",
            "exported-setup-props-interface",
            "exported-setup-props-interface-invalid-literal",
            "shared-line-type-exports",
            "shared-line-type-exports-invalid-literal",
        ],
    );
}
