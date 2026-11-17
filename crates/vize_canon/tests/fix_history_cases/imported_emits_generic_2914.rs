use crate::support::fix_history_runner::check_pack;

#[test]
fn imported_define_emits_type_retains_sfc_generic() {
    check_pack(
        "imported-emits-generic-2914",
        "fix_history_cases::imported_emits_generic_2914::imported_define_emits_type_retains_sfc_generic",
        "db8f7e52c7f337fdacb8f18be99aaf8d53f2cc2f",
        Some(2914),
        &["imported-emits-generic", "imported-emits-payload-error"],
    );
}
