use crate::support::fix_history_runner::check_pack;

#[test]
fn generated_public_instances_satisfy_vue_constraints_and_layout_excess_keys_report() {
    check_pack(
        "public-instance-contract-alignment-6027",
        "fix_history_cases::public_instance_contract_alignment_6027::generated_public_instances_satisfy_vue_constraints_and_layout_excess_keys_report",
        "d4c05ee736862e678ff8b7cac8a728d80f2193c0",
        Some(6027),
        &[
            "histoire-layout-matching-branches",
            "histoire-layout-branch-exclusive-fields",
            "public-instance-satisfies-component-public-instance",
            "public-instance-exact-emit-and-constraint-controls",
        ],
    );
}
