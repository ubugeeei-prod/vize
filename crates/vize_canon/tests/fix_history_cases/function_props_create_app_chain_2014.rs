use crate::support::fix_history_runner::check_pack;

#[test]
fn inline_function_props_are_contextually_typed() {
    check_pack(
        "function-props-create-app-chain-2014",
        "fix_history_cases::function_props_create_app_chain_2014::inline_function_props_are_contextually_typed",
        "cec7d50748ea10603b2dbf3242d832d01b7cfe5e",
        Some(2014),
        &[
            "inline-function-prop-and-create-app-chain",
            "inline-function-prop-in-v-if-branch",
            "inline-function-prop-wrong-parameter-use",
        ],
    );
}
