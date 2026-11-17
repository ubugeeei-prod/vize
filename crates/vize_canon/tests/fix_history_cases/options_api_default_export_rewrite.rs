use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_define_component_default_export_stays_valid() {
    check_pack(
        "options-api-default-export-rewrite",
        "fix_history_cases::options_api_default_export_rewrite::options_api_define_component_default_export_stays_valid",
        "48fd1ce241e33b85d57f54c969c27319edcfc3e7",
        None,
        &[
            "define-component-default-export",
            "define-component-default-export-wrong-name-type",
        ],
    );
}
