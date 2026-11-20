use crate::support::fix_history_runner::check_pack;

#[test]
fn imported_runtime_props_and_defaults_reach_setup_and_template() {
    check_pack(
        "imported-runtime-define-props-5962",
        "fix_history_cases::imported_runtime_define_props_5962::imported_runtime_props_and_defaults_reach_setup_and_template",
        "788000753c5c6bab73faedebc979a18d6db735e2",
        Some(5962),
        &[
            "imported-runtime-props-reach-script-and-template",
            "imported-runtime-number-prop-read-as-boolean",
            "with-defaults-resolves-imported-omit-defaults",
            "with-defaults-omitted-label-stays-optional",
            "runtime-props-cache-rebinds-local-names",
            "runtime-props-cache-rebound-label-wrong-type",
        ],
    );
}
