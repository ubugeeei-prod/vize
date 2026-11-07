//! Type-only exports are legal in `<script setup>` and must not receive the
//! runtime-export diagnostic used for unsupported value exports (#12073).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn setup_type_only_exports_have_no_modifier_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "setup-type-only-exports-12073",
        &[
            (
                "src/ExportedType.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/setup-type-exports-12073/ExportedType.vue.txt"
                ),
            ),
            (
                "src/ExportedInterfaceUnused.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/setup-type-exports-12073/ExportedInterfaceUnused.vue.txt"
                ),
            ),
            (
                "package.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/setup-type-exports-12073/package.json.txt"
                ),
            ),
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/setup-type-exports-12073/tsconfig.json.txt"
                ),
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(
        actual,
        Some(Vec::new()),
        "Vue permits exported type aliases and interfaces in script setup",
    );
}
