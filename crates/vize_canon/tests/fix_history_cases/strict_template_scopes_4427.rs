use crate::support::fix_history_runner::check_pack;

#[test]
fn strict_template_globals_keep_template_scope_aliases() {
    check_pack(
        "strict-template-scopes-4427",
        "fix_history_cases::strict_template_scopes_4427::strict_template_globals_keep_template_scope_aliases",
        "eb489b2579e9a9d92e5e7260c7eef32e2d701f56",
        Some(4427),
        &[
            "v-for-alias-before-script-length",
            "nested-v-for-aliases",
            "slot-props",
            "undeclared-names-in-template-scope",
        ],
    );
}
