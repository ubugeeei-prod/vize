use crate::support::fix_history_runner::check_pack;

#[test]
fn native_root_fallthrough_accepts_unknown_attrs() {
    check_pack(
        "open-native-root-fallthrough-4920",
        "fix_history_cases::open_native_root_fallthrough_4920::native_root_fallthrough_accepts_unknown_attrs",
        "09b4ddde76ed21942c0e302e7b2d4d8da289eaf1",
        Some(4920),
        &[
            "native-root-unknown-attr-ok",
            "native-root-unknown-attr-obligations",
            "generic-native-root-unknown-attr-ok",
            "generic-native-root-declared-prop-checks",
            "fallthrough-unknown-attrs-ok",
        ],
    );
}
