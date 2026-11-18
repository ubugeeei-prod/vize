use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_public_props_survive_an_imported_props_type() {
    check_pack(
        "options-api-props-beside-imported-props-6403",
        "fix_history_cases::options_api_props_beside_imported_props_6403::options_api_public_props_survive_an_imported_props_type",
        "beb84df46db611f6bf017fa6803af8cf84e6d59e",
        Some(6403),
        &[
            "array-props-beside-imported-props-type",
            "array-props-without-component-usage",
            "deferred-object-props-beside-imported-props-type",
            "deferred-object-props-wrong-type",
            "deferred-object-props-missing-required",
        ],
    );
}
