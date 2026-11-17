use crate::support::fix_history_runner::check_pack;

#[test]
fn define_props_reach_template_dollar_props() {
    check_pack(
        "template-dollar-props-4242",
        "fix_history_cases::template_dollar_props_4242::define_props_reach_template_dollar_props",
        "155555e58136758d84700e70a417a58588c0008d",
        Some(4242),
        &[
            "runtime-object-props-dollar-props",
            "runtime-object-props-dollar-props-valid",
            "type-only-props-dollar-props",
            "type-only-props-dollar-props-valid",
            "tsx-consumers-beside-dollar-props",
        ],
    );
}
