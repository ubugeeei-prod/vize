use crate::support::fix_history_runner::check_pack;

#[test]
fn closed_components_reject_unknown_fallthrough_attrs() {
    check_pack(
        "strict-unknown-component-attrs-4577",
        "fix_history_cases::strict_unknown_component_attrs_4577::closed_components_reject_unknown_fallthrough_attrs",
        "4b5cbe23cf33aca6b7f8aad1c3a17b365077cd75",
        Some(4577),
        &[
            "fallthrough-mono-ok",
            "fallthrough-data-aria-camel-ok",
            "fallthrough-mono-false-bad",
            "fallthrough-mono-false-native-binding-bad",
            "fallthrough-options-api-inherit-attrs-false-bad",
            "fallthrough-multi-bad",
            "fallthrough-multi-false-bad",
            "fallthrough-native-type-bad",
            "fallthrough-vif-both-mono-ok",
            "fallthrough-vif-both-mono-false-bad",
            "fallthrough-vif-mono-multi-bad",
            "fallthrough-root-vfor-bad",
            "fallthrough-template-root-vfor-bad",
            "fallthrough-vif-static-ok",
            "fallthrough-vif-elseif-static-ok",
            "fallthrough-vif-static-multi-bad",
            "fallthrough-vif-static-prop-ok",
            "fallthrough-component-root-open",
            "explicit-nested-attrs-forwarding-ok",
            "explicit-nested-attrs-forwarding-value-bad",
        ],
    );
}
