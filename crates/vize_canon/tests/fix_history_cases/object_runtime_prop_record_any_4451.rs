use crate::support::fix_history_runner::check_pack;

#[test]
fn object_runtime_props_infer_record_string_any() {
    check_pack(
        "object-runtime-prop-record-any-4451",
        "fix_history_cases::object_runtime_prop_record_any_4451::object_runtime_props_infer_record_string_any",
        "9a6dbdcfcfb33681ef5a310e841d73d6dd3e9875",
        Some(4451),
        &[
            "object-prop-untyped-member-access",
            "object-prop-assigned-to-string",
        ],
    );
}
