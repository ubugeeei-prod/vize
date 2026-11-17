use crate::support::fix_history_runner::check_pack;

#[test]
fn every_static_native_v_bind_is_checked_against_element_props() {
    check_pack(
        "native-element-v-bind-checks-3443",
        "fix_history_cases::native_element_v_bind_checks_3443::every_static_native_v_bind_is_checked_against_element_props",
        "51a13ef48cf2e509b39592c08017561fb5dd10ee",
        Some(3443),
        &[
            "string-href-and-tabindex",
            "number-href-and-tabindex",
            "vbind-wrong-type-fixture",
        ],
    );
}
