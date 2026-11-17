use crate::support::fix_history_runner::check_pack;

#[test]
fn explicit_ref_literals_survive_and_static_refs_are_not_reads() {
    check_pack(
        "ref-literal-and-static-ref-reads-5965",
        "fix_history_cases::ref_literal_and_static_ref_reads_5965::explicit_ref_literals_survive_and_static_refs_are_not_reads",
        "c0dae2112cf5ddf351946882af19b5ede2f76407",
        Some(5965),
        &[
            "explicit-ref-literal-reaches-literal-prop",
            "explicit-ref-literal-outside-prop-union",
            "static-template-refs-do-not-consume-locals",
            "static-template-ref-to-vue-ref-counts-as-use",
            "template-expression-read-consumes-ref-local",
            "template-expression-read-only-local",
        ],
    );
}
