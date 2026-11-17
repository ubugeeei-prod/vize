use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_method_diagnostics_anchor_on_authored_script_bytes() {
    check_pack(
        "options-api-method-body-anchors-3596",
        "fix_history_cases::options_api_method_body_anchors_3596::options_api_method_diagnostics_anchor_on_authored_script_bytes",
        "026f4b54f3fbc567af3a2351656f951681ff3149",
        Some(3596),
        &[
            "options-method-known-member",
            "options-method-missing-member-below-long-template",
        ],
    );
}
