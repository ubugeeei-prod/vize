use crate::support::fix_history_runner::check_pack;

#[test]
fn imported_intersection_props_are_exposed_to_templates() {
    check_pack(
        "imported-intersection-template-props-941",
        "fix_history_cases::imported_intersection_template_props_941::imported_intersection_props_are_exposed_to_templates",
        "cb6326e9375fa4d67cd677415bc9673524b13c46",
        Some(941),
        &[
            "imported-intersection-props-in-template",
            "defaulted-imported-prop-not-undefined",
            "imported-prop-wrong-member",
            "undeclared-template-name",
        ],
    );
}
