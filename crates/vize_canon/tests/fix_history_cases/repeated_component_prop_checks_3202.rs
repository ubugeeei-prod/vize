use crate::support::fix_history_runner::check_pack;

#[test]
fn repeated_component_prop_names_keep_unique_checks() {
    check_pack(
        "repeated-component-prop-checks-3202",
        "fix_history_cases::repeated_component_prop_checks_3202::repeated_component_prop_names_keep_unique_checks",
        "378683f0c1feee643cd1e01347ff8cc6d31141db",
        Some(3202),
        &[
            "static-and-bound-class",
            "static-and-bound-declared-prop",
            "repeated-declared-prop-wrong-bound-value",
        ],
    );
}
