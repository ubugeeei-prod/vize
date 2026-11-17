use crate::support::fix_history_runner::check_pack;

#[test]
fn no_space_default_export_is_wrapped_as_options_api() {
    check_pack(
        "no-space-default-export-3087",
        "fix_history_cases::no_space_default_export_3087::no_space_default_export_is_wrapped_as_options_api",
        "894b6f62b772b0a0e4e389bcc38cd9b8469d7557",
        Some(3087),
        &[
            "indented-no-space-default-export",
            "indented-no-space-default-export-template-error",
            "comment-boundary-default-export",
        ],
    );
}
