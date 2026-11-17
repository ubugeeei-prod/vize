use crate::support::fix_history_runner::check_pack;

#[test]
fn undefined_template_reference_anchors_at_its_own_occurrence() {
    check_pack(
        "undefined-template-ref-offsets-1283",
        "fix_history_cases::undefined_template_ref_offsets_1283::undefined_template_reference_anchors_at_its_own_occurrence",
        "938a1b11ad3cab1e103544e7c0ef1e56abd7a3c6",
        Some(1283),
        &[
            "member-and-binding-share-name-defined",
            "undefined-name-after-same-named-member",
            "historical-js-setup-dom-global-name",
        ],
    );
}
