use crate::support::fix_history_runner::check_pack;

#[test]
fn art_variant_template_reads_count_as_setup_binding_usage() {
    check_pack(
        "art-variant-bindings-no-unused-1905",
        "fix_history_cases::art_variant_bindings_no_unused_1905::art_variant_template_reads_count_as_setup_binding_usage",
        "21d5dd73da450a3b3fb0638ef8544798ccc11e25",
        Some(1905),
        &[
            "art-variant-uses-schema-and-handler",
            "art-variant-leaves-handler-unused",
        ],
    );
}
