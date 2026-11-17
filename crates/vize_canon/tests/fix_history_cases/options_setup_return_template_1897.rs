use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_setup_returns_reach_the_template_unwrapped() {
    check_pack(
        "options-setup-return-template-1897",
        "fix_history_cases::options_setup_return_template_1897::options_api_setup_returns_reach_the_template_unwrapped",
        "b46bd454b06c70329032e98bee050d80f877c2d5",
        Some(1897),
        &[
            "setup-ref-return-unwrapped",
            "setup-ref-return-wrong-argument",
            "setup-ref-and-object-returns",
            "setup-object-return-wrong-member",
        ],
    );
}
