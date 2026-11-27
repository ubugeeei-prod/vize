use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

/// A generic `<script setup>` component shipped as `.d.ts` carries its
/// listener props in `__ctx.props` with Vue's `EmitsToProps` spelling, which
/// keeps an authored hyphen (`onRow-clicked`). An authored `@row-clicked`
/// listener reads that key, so its payload is the declared event object, not
/// `any`. A camel emit of the same component, a non-generic packaged component
/// and a local generic SFC with the same kebab emit keep their payloads.
#[test]
fn kebab_listeners_of_packaged_generic_components_read_the_hyphenated_prop() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "library-generic-kebab-emits",
        &[
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/library-generic-kebab-emits/tsconfig.json.txt"
                ),
            ),
            (
                "src/lib/table.d.ts",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/library-generic-kebab-emits/table.d.ts.txt"
                ),
            ),
            (
                "src/lib/plain.d.ts",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/library-generic-kebab-emits/plain.d.ts.txt"
                ),
            ),
            (
                "src/Local.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/library-generic-kebab-emits/Local.vue.txt"
                ),
            ),
            (
                "src/App.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/library-generic-kebab-emits/App.vue.txt"
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
        vec![
            (
                "src/App.vue",
                Some(2345),
                "12:54:error Argument of type 'RowEvent<Row>' is not assignable to parameter of type 'never'.",
            ),
            (
                "src/App.vue",
                Some(2345),
                "13:57:error Argument of type 'Row' is not assignable to parameter of type 'never'.",
            ),
            (
                "src/App.vue",
                Some(2345),
                "14:48:error Argument of type 'Row' is not assignable to parameter of type 'never'.",
            ),
            (
                "src/App.vue",
                Some(2345),
                "15:55:error Argument of type 'RowEvent<string>' is not assignable to parameter of type 'never'.",
            ),
            (
                "src/App.vue",
                Some(2345),
                "16:54:error Argument of type 'RowEvent<Row>' is not assignable to parameter of type 'never'.",
            ),
        ]
    );
}
