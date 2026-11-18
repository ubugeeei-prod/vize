use crate::support::fix_history_runner::check_pack;

#[test]
fn with_defaults_reexported_interface_props_and_as_assertions() {
    check_pack(
        "with-defaults-reexported-props-as-assertion-952",
        "fix_history_cases::with_defaults_reexported_props_as_assertion_952::with_defaults_reexported_interface_props_and_as_assertions",
        "80333546c87b9bbd40362926e61114551eb28f93",
        Some(952),
        &[
            "with-defaults-direct-template-props",
            "with-defaults-missing-default-keeps-undefined",
            "reexported-vue-interface-template-props",
            "reexported-vue-interface-wrong-prop-type",
            "mixed-reexported-vue-interface-template-props",
            "mixed-reexported-vue-interface-wrong-prop-type",
            "as-assertions-with-prop-named-as",
            "as-assertion-bad-conversion-with-prop-named-as",
        ],
    );
}
