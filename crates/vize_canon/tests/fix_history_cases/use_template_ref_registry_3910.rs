use crate::support::fix_history_runner::check_pack;

#[test]
fn use_template_ref_resolves_static_template_refs() {
    check_pack(
        "use-template-ref-registry-3910",
        "fix_history_cases::use_template_ref_registry_3910::use_template_ref_resolves_static_template_refs",
        "a0c4bee37d78398ea20d244945a2a912cddfb505",
        Some(3910),
        &[
            "explicit-generic-template-ref-member-errors",
            "registered-and-ref-bound-template-refs",
            "registered-div-ref-optional-chain",
            "registered-div-ref-possibly-null",
            "svg-anchor-ref-uses-svg-tag-map",
            "svg-anchor-ref-href-is-not-string",
        ],
    );
}
