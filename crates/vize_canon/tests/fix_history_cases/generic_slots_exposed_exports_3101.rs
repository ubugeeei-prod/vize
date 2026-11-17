use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_sfc_type_exports_keep_their_type_parameters() {
    check_pack(
        "generic-slots-exposed-exports-3101",
        "fix_history_cases::generic_slots_exposed_exports_3101::generic_sfc_type_exports_keep_their_type_parameters",
        "30db60efb71eed8dd060b373d50a29e137f43c46",
        Some(3101),
        &[
            "generic-slot-and-exposed-components",
            "generic-type-exports-consumer",
            "generic-type-exports-consumer-wrong-slot-value",
            "generic-components-template-consumer",
            "generic-components-template-constraint-violation",
        ],
    );
}
