use crate::support::fix_history_runner::check_pack;

#[test]
fn multiline_native_v_bind_anchors_at_the_attribute_name() {
    check_pack(
        "multiline-native-binding-anchor-3563",
        "fix_history_cases::multiline_native_binding_anchor_3563::multiline_native_v_bind_anchors_at_the_attribute_name",
        "7424ea068468a631bfdca4ba1f408310d7231ca2",
        Some(3563),
        &[
            "multiline-native-ref-callback",
            "multiline-native-disabled-boolean",
            "multiline-native-disabled-wrong-type",
        ],
    );
}
