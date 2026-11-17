use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_emit_overload_listener_props_stay_within_depth() {
    check_pack(
        "generic-emit-overload-depth-3150",
        "fix_history_cases::generic_emit_overload_depth_3150::generic_emit_overload_listener_props_stay_within_depth",
        "5257a671727fc55e94bb227247269d23a8eda13e",
        Some(3150),
        &[
            "generic-emit-overload-expansion",
            "generic-emit-overload-extra-argument",
        ],
    );
}
