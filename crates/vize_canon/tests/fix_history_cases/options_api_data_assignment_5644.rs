use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_data_assignments_are_mutable_from_template_expressions() {
    check_pack(
        "options-api-data-assignment-5644",
        "fix_history_cases::options_api_data_assignment_5644::options_api_data_assignments_are_mutable_from_template_expressions",
        "024b4b712cb03528547b03cbb01cf459ca36fa08",
        Some(5644),
        &[
            "data-template-assignment",
            "data-template-assignment-wrong-type",
            "computed-template-assignment-stays-const",
        ],
    );
}
