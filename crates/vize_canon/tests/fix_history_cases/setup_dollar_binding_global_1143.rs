use crate::support::fix_history_runner::check_pack;

#[test]
fn setup_dollar_bindings_are_not_redeclared_as_instance_globals() {
    check_pack(
        "setup-dollar-binding-global-1143",
        "fix_history_cases::setup_dollar_binding_global_1143::setup_dollar_bindings_are_not_redeclared_as_instance_globals",
        "f30af4cfaf242e826d7520726019467959c31ff0",
        Some(1143),
        &[
            "historical-any-dollar-binding-guard",
            "typed-dollar-binding-member",
            "typed-dollar-binding-missing-member",
        ],
    );
}
