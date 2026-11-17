use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_type_cast_props_defer_to_setup_value_scope() {
    check_pack(
        "options-api-cast-props-value-scope",
        "fix_history_cases::options_api_cast_props_value_scope::options_api_type_cast_props_defer_to_setup_value_scope",
        "921ee580f2548f4e97d3d208b2c1a457a28da306",
        None,
        &[
            "type-cast-props-script-only",
            "type-cast-props-template-use",
            "type-cast-props-template-unknown-member",
        ],
    );
}
