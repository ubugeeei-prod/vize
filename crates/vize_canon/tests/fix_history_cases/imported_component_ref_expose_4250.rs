use crate::support::fix_history_runner::check_pack;

#[test]
fn imported_component_refs_resolve_exposed_members() {
    check_pack(
        "imported-component-ref-expose-4250",
        "fix_history_cases::imported_component_ref_expose_4250::imported_component_refs_resolve_exposed_members",
        "6aef3e8822e398c25fc3409b2c6192a77fe2c51c",
        Some(4250),
        &[
            "repaired-expose-members-clean",
            "absent-and-private-members-and-prop-controls",
        ],
    );
}
