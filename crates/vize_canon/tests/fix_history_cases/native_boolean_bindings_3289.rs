use crate::support::fix_history_runner::check_pack;

#[test]
fn native_boolean_bindings_are_checked_against_native_elements() {
    check_pack(
        "native-boolean-bindings-3289",
        "fix_history_cases::native_boolean_bindings_3289::native_boolean_bindings_are_checked_against_native_elements",
        "3bd325f3b5a311eb37242b403a94876e4727fb01",
        Some(3289),
        &["boolean-disabled-binding", "string-disabled-binding"],
    );
}
