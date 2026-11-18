use crate::support::fix_history_runner::check_pack;

#[test]
fn strict_unknown_props_accept_html_attrs_and_report_widened_literals_once() {
    check_pack(
        "strict-html-fallthrough-literal-dedup-4992",
        "fix_history_cases::strict_html_fallthrough_literal_dedup_4992::strict_unknown_props_accept_html_attrs_and_report_widened_literals_once",
        "13bcf1ba4ddb588b79296623f259c310a1bab2b5",
        Some(4992),
        &[
            "real-html-attrs-on-closed-component",
            "unknown-attr-stays-strict-invalid",
            "boolean-literal-binding-valid",
            "number-literal-binding-reports-once-invalid",
        ],
    );
}
