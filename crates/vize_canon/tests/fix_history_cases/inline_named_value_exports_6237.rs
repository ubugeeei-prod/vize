use crate::support::fix_history_runner::check_pack;

#[test]
fn inline_plain_script_value_exports_keep_authored_bytes() {
    check_pack(
        "inline-named-value-exports-6237",
        "fix_history_cases::inline_named_value_exports_6237::inline_plain_script_value_exports_keep_authored_bytes",
        "2b0a8604c1d33afa766153b9fcb11292b9ecf95e",
        Some(6237),
        &[
            "inline-exports-lf-consumer",
            "inline-exports-crlf-consumer",
            "inline-exports-consumer-wrong-type",
            "export-keywords-in-strings-comments-and-multiline",
            "multiline-export-consumer-wrong-type",
        ],
    );
}
