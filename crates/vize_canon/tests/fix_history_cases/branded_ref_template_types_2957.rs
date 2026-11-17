use crate::support::fix_history_runner::check_pack;

#[test]
fn branded_ref_template_unwrap_preserves_brand() {
    check_pack(
        "branded-ref-template-types-2957",
        "fix_history_cases::branded_ref_template_types_2957::branded_ref_template_unwrap_preserves_brand",
        "145baca6ae3b4a45b847b7dcbf2cbcc39cd1e72d",
        Some(2957),
        &[
            "branded-string-ref-argument",
            "branded-number-ref-argument",
            "plain-string-ref-rejected-by-brand",
        ],
    );
}
