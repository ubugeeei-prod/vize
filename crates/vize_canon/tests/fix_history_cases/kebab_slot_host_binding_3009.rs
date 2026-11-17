use crate::support::fix_history_runner::check_pack;

#[test]
fn kebab_slot_hosts_resolve_pascal_setup_bindings() {
    check_pack(
        "kebab-slot-host-binding-3009",
        "fix_history_cases::kebab_slot_host_binding_3009::kebab_slot_hosts_resolve_pascal_setup_bindings",
        "46a2340e53595c53829320bdcf6a62c8ebdad65f",
        Some(3009),
        &[
            "kebab-slot-host-typed-slot-props",
            "kebab-slot-host-missing-member",
            "nested-kebab-slot-host-missing-member",
        ],
    );
}
