use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_nested_v_for_member_array_aliases_keep_element_types() {
    check_pack(
        "legacy-nested-vfor-member-arrays-2259",
        "fix_history_cases::legacy_nested_vfor_member_arrays_2259::legacy_nested_v_for_member_array_aliases_keep_element_types",
        "dfc0f23a768488cd504346a2b07d0ccef3a85ee5",
        Some(2259),
        &["nested-member-array-aliases-typed"],
    );
}
