use crate::support::fix_history_runner::check_pack;

#[test]
fn scoped_slot_props_typed_from_child_define_slots() {
    check_pack(
        "scoped-slot-props-define-slots-764",
        "fix_history_cases::scoped_slot_props_define_slots_764::scoped_slot_props_typed_from_child_define_slots",
        "76132582fd93607bb43429918bdd1dcff68b1fa9",
        Some(764),
        &[
            "slot-prop-used-as-number",
            "slot-prop-misused-as-string",
            "slot-prop-forwarded-to-component-prop",
            "slot-prop-forwarded-to-wrong-component-prop",
        ],
    );
}
