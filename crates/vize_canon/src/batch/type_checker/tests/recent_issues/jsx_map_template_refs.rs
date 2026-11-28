use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

/// A component without a template renders its script's JSX. A `ref` on an
/// element of an `items.map()` child lowers to `v-for` with `ref_for`, so
/// `useTemplateRef` holds an array of the elements, as a template `v-for` ref
/// does. A key the JSX never registers keeps Vue's untyped result, unchecked.
#[test]
fn jsx_map_refs_type_use_template_ref_as_an_element_array() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "jsx-map-template-refs",
        &[
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/jsx-map-template-refs/tsconfig.json.txt"
                ),
            ),
            (
                "src/App.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/jsx-map-template-refs/App.vue.txt"
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
                Some(2322),
                "12:11:error Type 'HTMLLIElement[] | null' is not assignable to type 'HTMLLIElement | null'.\nType 'HTMLLIElement[]' is missing the following properties from type 'HTMLLIElement': type, value, addEventListener, removeEventListener, and 320 more.",
            ),
            (
                "src/App.vue",
                Some(2322),
                "14:11:error Type 'unknown' is not assignable to type 'string'.",
            ),
        ]
    );
}
