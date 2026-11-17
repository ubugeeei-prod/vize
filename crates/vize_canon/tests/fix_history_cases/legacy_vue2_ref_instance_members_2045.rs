use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_component_refs_include_instance_surface() {
    check_pack(
        "legacy-vue2-ref-instance-members-2045",
        "fix_history_cases::legacy_vue2_ref_instance_members_2045::legacy_vue2_component_refs_include_instance_surface",
        "71f9a8115e9c3cc70cafe600ed82063509353327",
        Some(2045),
        &[
            "component-ref-instance-surface",
            "ref-prop-wrong-annotation",
            "ref-el-is-element",
        ],
    );
}
