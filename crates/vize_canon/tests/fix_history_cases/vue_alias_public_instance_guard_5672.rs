use crate::support::fix_history_runner::check_pack;

#[test]
fn vue_alias_without_public_instance_degrades_template_context() {
    check_pack(
        "vue-alias-public-instance-guard-5672",
        "fix_history_cases::vue_alias_public_instance_guard_5672::vue_alias_without_public_instance_degrades_template_context",
        "c220f47b65134eee36851c7841b1e8d5567eafe4",
        Some(5672),
        &[
            "standard-vue-template-context",
            "alias-without-public-instance",
            "alias-without-public-instance-wrong-assignment",
        ],
    );
}
