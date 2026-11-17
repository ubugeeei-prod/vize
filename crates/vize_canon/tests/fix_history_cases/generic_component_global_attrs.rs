use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_closed_component_accepts_global_html_attrs() {
    check_pack(
        "generic-component-global-attrs",
        "fix_history_cases::generic_component_global_attrs::generic_closed_component_accepts_global_html_attrs",
        "d7796777839f961aa97d1a55397ad4e2d0372a31",
        None,
        &[
            "closed-generic-global-html-attrs",
            "closed-generic-unknown-attrs-rejected",
        ],
    );
}
