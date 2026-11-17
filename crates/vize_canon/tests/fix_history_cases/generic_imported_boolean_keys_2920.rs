use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_props_resolve_imported_boolean_keys() {
    check_pack(
        "generic-imported-boolean-keys-2920",
        "fix_history_cases::generic_imported_boolean_keys_2920::generic_props_resolve_imported_boolean_keys",
        "37cb5631062a8926497835844f8d41db680128eb",
        Some(2920),
        &[
            "imported-heritage-boolean-key",
            "imported-and-local-boolean-keys",
            "imported-boolean-key-type-error",
        ],
    );
}
