use crate::support::fix_history_runner::check_pack;

#[test]
fn v_bind_object_spread_is_checked_against_child_props() {
    check_pack(
        "v-bind-object-prop-spread-3526",
        "fix_history_cases::v_bind_object_prop_spread_3526::v_bind_object_spread_is_checked_against_child_props",
        "21e5bd408bfdf22f0c043f1e0c50d92f4055479a",
        Some(3526),
        &[
            "correct-spread",
            "wrong-and-missing-spreads",
            "spread-beside-correct-named-prop",
            "spread-beside-wrong-named-prop",
            "error-inside-spread-expression",
        ],
    );
}
