use crate::support::fix_history_runner::check_pack;

#[test]
fn fallthrough_attrs_follow_inherit_attrs_and_root_shape() {
    check_pack(
        "strict-fallthrough-root-shape-4491",
        "fix_history_cases::strict_fallthrough_root_shape_4491::fallthrough_attrs_follow_inherit_attrs_and_root_shape",
        "cf759a1ee4e6f2c557f87a892a85112960ca65d9",
        Some(4491),
        &[
            "mono-root-ok",
            "mono-root-wrong-title",
            "mono-root-inherit-attrs-false-rejects-id",
            "mono-root-inherit-attrs-false-rejects-native-binding",
            "options-api-inherit-attrs-false-rejects-id",
            "multi-root-rejects-id",
            "multi-root-inherit-attrs-false-rejects-id",
            "native-root-attr-type-mismatch",
            "vif-both-mono-ok",
            "vif-both-mono-inherit-attrs-false-rejects-id",
            "vif-mono-else-multi-rejects-id",
            "root-vfor-rejects-id",
            "template-root-vfor-rejects-id",
            "vif-static-true-ok",
            "vif-elseif-static-true-ok",
            "vif-static-true-multi-rejects-id",
            "vif-static-true-prop-ok",
            "single-component-root-forwarding-open",
        ],
    );
}
