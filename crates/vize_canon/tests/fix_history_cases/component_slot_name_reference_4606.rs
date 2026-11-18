use crate::support::fix_history_runner::check_pack;

#[test]
fn static_component_slot_names_resolve_against_define_slots() {
    check_pack(
        "component-slot-name-reference-4606",
        "fix_history_cases::component_slot_name_reference_4606::static_component_slot_names_resolve_against_define_slots",
        "ce8eae99b0981377f73239429d6d65896b59f371",
        Some(4606),
        &[
            "declared-default-slot",
            "undeclared-slot-name",
            "default-slot-payload-misuse-invalid",
        ],
    );
}
