use crate::support::fix_history_runner::check_pack;

#[test]
fn type_position_imports_are_not_value_anchored() {
    check_pack(
        "type-only-import-value-anchors",
        "fix_history_cases::type_only_import_value_anchors::type_position_imports_are_not_value_anchored",
        "f37a2d6764892f711d77f3cdf1c57d75a067e735",
        None,
        &[
            "type-only-imports-in-indexed-alias",
            "type-only-import-missing-indexed-key",
            "typeof-root-value-import",
        ],
    );
}
