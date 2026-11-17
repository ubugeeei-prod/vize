use crate::support::fix_history_runner::check_pack;

#[test]
fn ambient_global_components_type_authored_callbacks() {
    check_pack(
        "ambient-global-callback-props-3812",
        "fix_history_cases::ambient_global_callback_props_3812::ambient_global_components_type_authored_callbacks",
        "c6f498cff95dd574e2d9741989fec1c9fcdaed4b",
        Some(3812),
        &[
            "ambient-array-callbacks-contextually-typed",
            "ambient-array-callbacks-valid",
            "ambient-event-callbacks-contextually-typed",
            "ambient-event-callbacks-valid",
            "unknown-component-array-callback-implicit-any",
        ],
    );
}
