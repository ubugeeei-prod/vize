use crate::support::fix_history_runner::check_pack;

#[test]
fn independent_ambient_setup_declarations_preserve_batch_diagnostics() {
    check_pack(
        "setup-ambient-declarations-6170",
        "fix_history_cases::setup_ambient_declarations_6170::independent_ambient_setup_declarations_preserve_batch_diagnostics",
        "71451516ec37ed6ae600ba09f57f5d452961645a",
        Some(6170),
        &[
            "use-before-declaration",
            "implicit-any",
            "const-assignment",
            "string-property-typo",
            "union-property",
            "utf16-prefix-position",
            "overloads",
            "ts-ignore",
            "ambient-initializer",
            "template-property-typo",
            "unused-expect-error",
            "overload-mismatch",
        ],
    );
}
