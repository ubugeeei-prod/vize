use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_public_instance_members_reach_script_and_ref_consumers() {
    check_pack(
        "options-api-public-instance-members-4251",
        "fix_history_cases::options_api_public_instance_members_4251::options_api_public_instance_members_reach_script_and_ref_consumers",
        "97898a1a07aa4fd4d3f3ab413e8852ef1b541ec8",
        Some(4251),
        &[
            "source-lane-project",
            "source-lane-member-matrix",
            "source-lane-script-consumer-wrong-member-type",
            "options-template-data-binding",
            "options-template-data-binding-wrong-member",
        ],
    );
}
