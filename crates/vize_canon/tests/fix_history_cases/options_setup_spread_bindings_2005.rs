use crate::support::fix_history_runner::check_pack;

#[test]
fn options_setup_return_spread_bindings_reach_the_template() {
    check_pack(
        "options-setup-spread-bindings-2005",
        "fix_history_cases::options_setup_spread_bindings_2005::options_setup_return_spread_bindings_reach_the_template",
        "cd1fc1447ab6b7d4e0a5afb589b0bafdbc1714a9",
        Some(2005),
        &[
            "setup-spread-local-stub-helpers",
            "setup-spread-vue-to-refs",
            "setup-spread-binding-wrong-type",
            "setup-spread-binding-misspelled",
        ],
    );
}
