use crate::support::fix_history_runner::check_pack;

#[test]
fn unnamed_inner_slot_keeps_outer_scoped_slot_payload() {
    check_pack(
        "unnamed-inner-slot-outer-payload-5693",
        "fix_history_cases::unnamed_inner_slot_outer_payload_5693::unnamed_inner_slot_keeps_outer_scoped_slot_payload",
        "8e8ec535688244d062f15880e2e1295a862c8795",
        Some(5693),
        &[
            "outer-slot-props-through-unnamed-inner-slot",
            "outer-slot-index-passed-as-string-invalid",
        ],
    );
}
