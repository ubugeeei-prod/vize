use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_sfc_props_infer_through_extracted_component_props() {
    check_pack(
        "generic-sfc-extracted-props-2701",
        "fix_history_cases::generic_sfc_extracted_props_2701::generic_sfc_props_infer_through_extracted_component_props",
        "89131c4b4b3a605f5c12c771dccb54fe91d38482",
        Some(2701),
        &[
            "generic-default-does-not-specialize-extracted-props",
            "concrete-undefined-prop-type-error",
        ],
    );
}
