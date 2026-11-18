use crate::support::fix_history_runner::check_pack;

#[test]
fn undefined_template_ref_anchors_bare_identifier() {
    check_pack(
        "undefined-template-ref-anchor-1284",
        "fix_history_cases::undefined_template_ref_anchor_1284::undefined_template_ref_anchors_bare_identifier",
        "eb62d44acada562ca55ed5da9d26b10c348e90c0",
        Some(1284),
        &["member-read-only", "undefined-after-same-named-member"],
    );
}
