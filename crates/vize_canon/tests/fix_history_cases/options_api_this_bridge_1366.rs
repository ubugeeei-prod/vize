use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_this_resolves_across_setup_data_computed_methods() {
    check_pack(
        "options-api-this-bridge-1366",
        "fix_history_cases::options_api_this_bridge_1366::options_api_this_resolves_across_setup_data_computed_methods",
        "b01ecf39085b5d14a01335749ddc7178d3b894d7",
        Some(1366),
        &[
            "options-api-this-across-blocks",
            "options-api-template-method-wrong-argument",
            "options-api-method-body-wrong-data-assignment",
        ],
    );
}
