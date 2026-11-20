use crate::support::fix_history_runner::check_pack;

#[test]
fn project_global_components_are_typed_from_ambient_declarations() {
    check_pack(
        "project-global-component-props-1153",
        "fix_history_cases::project_global_component_props_1153::project_global_components_are_typed_from_ambient_declarations",
        "7000ca00f400b745df16ac6304138eb5fcc01f4b",
        Some(1153),
        &[
            "historical-global-prop-mismatch",
            "global-prop-and-event-valid",
            "global-event-payload-mismatch",
        ],
    );
}
