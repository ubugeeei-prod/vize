use crate::support::fix_history_runner::check_pack;

#[test]
fn exposed_template_ref_unwraps_on_component_instance() {
    check_pack(
        "exposed-template-ref-unwrap-2632",
        "fix_history_cases::exposed_template_ref_unwrap_2632::exposed_template_ref_unwraps_on_component_instance",
        "745bc7c6301766b462bc8f5a9ba3104656c5c2a6",
        Some(2632),
        &[
            "exposed-template-ref-unwrapped",
            "exposed-template-ref-missing-member-error",
        ],
    );
}
