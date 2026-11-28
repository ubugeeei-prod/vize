use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

/// Under `noPropertyAccessFromIndexSignature`, a listener for an event the
/// component does not declare (`@click`, `@blur`, a kebab `@focus-in`) is a
/// fallthrough listener, as vue-tsc reads it: its navigation reference must
/// not report `TS4111`. A declared event keeps its handler check.
#[test]
fn undeclared_listeners_stay_clean_under_no_property_access_from_index_signature() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "undeclared-listener-index-signature",
        &[
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/undeclared-listener-index-signature/tsconfig.json.txt"
                ),
            ),
            (
                "src/Child.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/undeclared-listener-index-signature/Child.vue.txt"
                ),
            ),
            (
                "src/Parent.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/undeclared-listener-index-signature/Parent.vue.txt"
                ),
            ),
        ],
    );
    let diagnostics = snapshot_project_diagnostics(&root).unwrap();
    std::fs::remove_dir_all(root).unwrap();
    let diagnostics: Vec<_> = diagnostics
        .iter()
        .map(|(file, code, message)| (file.as_str(), *code, message.as_str()))
        .collect();
    assert_eq!(
        diagnostics,
        vec![(
            "src/Parent.vue",
            Some(2322),
            "12:21:error Type '(id: string) => string' is not assignable to type '(id: number) => any'.\nTypes of parameters 'id' and 'id' are incompatible.\nType 'number' is not assignable to type 'string'.",
        )]
    );
}
