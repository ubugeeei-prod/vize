use crate::support::fix_history_runner::check_pack;

#[test]
fn setup_props_read_by_template_is_not_unused() {
    check_pack(
        "setup-props-shadow-anchor-3934",
        "fix_history_cases::setup_props_shadow_anchor_3934::setup_props_read_by_template_is_not_unused",
        "7c83dd0193b06a4cee3a1825a374ca0cd11fbd85",
        Some(3934),
        &[
            "template-v-bind-props-read",
            "template-without-props-reference-unused",
        ],
    );
}
