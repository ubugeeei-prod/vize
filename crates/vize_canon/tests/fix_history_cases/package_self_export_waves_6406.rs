use crate::support::fix_history_runner::check_pack;

#[test]
fn package_self_exports_resolve_in_later_route_waves() {
    check_pack(
        "package-self-export-waves-6406",
        "fix_history_cases::package_self_export_waves_6406::package_self_exports_resolve_in_later_route_waves",
        "3a2fd80a314ae1c644af30bed1419855b2133c1e",
        Some(6406),
        &["self-export-valid", "self-export-invalid"],
    );
}
