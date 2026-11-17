use crate::support::fix_history_runner::check_pack;

#[test]
fn macro_type_dependencies_resolve_in_scope_and_vfor_keeps_literals() {
    check_pack(
        "macro-type-scope-vfor-literals-6254",
        "fix_history_cases::macro_type_scope_vfor_literals_6254::macro_type_dependencies_resolve_in_scope_and_vfor_keeps_literals",
        "0bd36715d4ecbb448630157c7ce233b60e142362",
        Some(6254),
        &[
            "setup-scoped-macro-types-and-usages",
            "setup-scoped-macro-types-parent-mismatch",
            "setup-scoped-class-prop-member-typo",
            "vfor-literal-array-keeps-literal-items",
            "vfor-literal-array-item-outside-parameter",
        ],
    );
}
