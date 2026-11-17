use crate::support::fix_history_runner::check_pack;

#[test]
fn split_script_named_exports_resolve_through_extended_alias() {
    check_pack(
        "split-script-named-exports-extended-alias",
        "fix_history_cases::split_script_named_exports_extended_alias::split_script_named_exports_resolve_through_extended_alias",
        "11f3a6563c48d38997b2e603d282bbaaa9b53d2c",
        None,
        &[
            "split-script-named-exports-imported",
            "split-script-named-export-type-checked",
            "split-script-export-default-with-named-exports",
            "named-exports-through-extended-paths-alias",
            "named-export-through-extended-alias-type-checked",
        ],
    );
}
