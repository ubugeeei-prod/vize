use crate::support::fix_history_runner::check_pack;

#[test]
fn setup_return_spread_supplies_dollar_name_without_instance_global_redeclaration() {
    check_pack(
        "nuxt2-setup-spread-instance-global-3698",
        "fix_history_cases::nuxt2_setup_spread_instance_global_3698::setup_return_spread_supplies_dollar_name_without_instance_global_redeclaration",
        "6262435c1ac66de825c2862adb88f60ce952d5d1",
        Some(3698),
        &[
            "setup-spread-dollar-fetch",
            "setup-spread-handler-wrong-arity",
            "no-spread-dollar-fetch-instance-global",
            "vue-typed-spread-handler-wrong-arity",
            "vue-typed-no-spread-dollar-fetch",
        ],
    );
}
