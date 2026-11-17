use crate::support::fix_history_runner::check_pack;

#[test]
fn v_for_over_object_types_key_as_keyof_and_value_as_member() {
    check_pack(
        "v-for-object-key-keyof-767",
        "fix_history_cases::v_for_object_key_keyof_767::v_for_over_object_types_key_as_keyof_and_value_as_member",
        "b06aeecfb8c215de44dcfbce1978c5ede4f8ac41",
        Some(767),
        &[
            "object-source-key-and-value",
            "object-source-key-not-number",
            "object-source-value-not-string",
        ],
    );
}
