use crate::support::fix_history_runner::check_pack;

#[test]
fn generated_default_export_alias_is_not_reported_unused() {
    check_pack(
        "default-export-alias-unused-1949",
        "fix_history_cases::default_export_alias_unused_1949::generated_default_export_alias_is_not_reported_unused",
        "958a2ac4c623225bd031a132de8540be8fd78f82",
        Some(1949),
        &[
            "options-default-export-no-unused-locals",
            "authored-unused-local-still-reported",
        ],
    );
}
