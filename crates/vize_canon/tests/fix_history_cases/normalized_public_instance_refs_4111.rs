use crate::support::fix_history_runner::check_pack;

#[test]
fn template_refs_see_the_normalized_public_instance() {
    check_pack(
        "normalized-public-instance-refs-4111",
        "fix_history_cases::normalized_public_instance_refs_4111::template_refs_see_the_normalized_public_instance",
        "e96023e4393b0301a2e6a76621053202676ff2d5",
        Some(4111),
        &[
            "template-ref-public-instance-strict-emits",
            "template-ref-public-instance-wrong-emit-payload",
            "template-ref-historical-generic-instance-consumer",
        ],
    );
}
