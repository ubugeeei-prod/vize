use crate::support::fix_history_runner::check_pack;

#[test]
fn plain_script_enum_class_identity_and_vue_declaration_imports() {
    check_pack(
        "plain-script-public-declarations-6245",
        "fix_history_cases::plain_script_public_declarations_6245::plain_script_enum_class_identity_and_vue_declaration_imports",
        "95401a66090e358f72063d7c4bd73c2b70b593b7",
        Some(6245),
        &[
            "enum-class-exports-keep-identity",
            "enum-class-exports-reject-structural-values",
            "inline-export-wrong-value-type",
            "vue-declaration-without-source-sfc",
            "vue-declaration-without-source-sfc-wrong-type",
            "missing-sfc-and-declaration",
        ],
    );
}
