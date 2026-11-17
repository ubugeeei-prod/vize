use crate::support::fix_history_runner::check_pack;

#[test]
fn component_fallthrough_touch_listener_stays_loose() {
    check_pack(
        "component-fallthrough-touch-listener-2732",
        "fix_history_cases::component_fallthrough_touch_listener_2732::component_fallthrough_touch_listener_stays_loose",
        "b41d999468128f920fcbbda1cc5c019e90dc75ee",
        Some(2732),
        &[
            "component-fallthrough-touch-listener",
            "native-element-touch-listener-type-error",
        ],
    );
}
