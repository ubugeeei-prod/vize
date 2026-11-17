use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_props_reach_template_dollar_props() {
    check_pack(
        "options-template-dollar-props-4242",
        "fix_history_cases::options_template_dollar_props_4242::options_api_props_reach_template_dollar_props",
        "155555e58136758d84700e70a417a58588c0008d",
        Some(4242),
        &["options-api-dollar-props", "options-api-dollar-props-valid"],
    );
}
