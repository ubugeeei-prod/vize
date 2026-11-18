use crate::support::fix_history_runner::check_pack;

#[test]
fn unresolved_slot_host_uses_ambient_global_component_slots() {
    check_pack(
        "ambient-global-slot-host-3011",
        "fix_history_cases::ambient_global_slot_host_3011::unresolved_slot_host_uses_ambient_global_component_slots",
        "22a49aca2fb9331217d59511fbd6cf4f2414cd04",
        Some(3011),
        &[
            "unresolved-slot-host-any-fallback",
            "ambient-pascal-slot-host-typed-slot",
            "ambient-pascal-slot-host-missing-member",
        ],
    );
}
