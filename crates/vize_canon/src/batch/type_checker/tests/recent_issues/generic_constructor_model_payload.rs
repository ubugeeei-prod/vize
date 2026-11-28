use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

/// A constructor intersection whose generic signature is not the first, or a
/// lone generic constructor without `$emit`, is constructed with the bound
/// props, as vue-tsc's `new Component(props)` does. Its `update:modelValue`
/// payload is then the bound `modelValue` type instead of the erased
/// `unknown`, under a `v-if` too, while a non-generic constructor keeps its
/// declared payload.
#[test]
fn generic_constructor_listeners_read_the_props_bound_instance() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "generic-constructor-model-payload",
        &[
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/generic-constructor-model-payload/tsconfig.json.txt"
                ),
            ),
            (
                "src/App.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/generic-constructor-model-payload/App.vue.txt"
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
                "25:61:error Argument of type 'string[]' is not assignable to parameter of type 'number'.",
            ),
            (
                "src/App.vue",
                Some(2345),
                "26:84:error Argument of type 'string[]' is not assignable to parameter of type 'number'.",
            ),
            (
                "src/App.vue",
                Some(2345),
                "27:60:error Argument of type 'string[]' is not assignable to parameter of type 'number'.",
            ),
            (
                "src/App.vue",
                Some(2345),
                "28:55:error Argument of type 'number' is not assignable to parameter of type 'string[]'.",
            ),
        ]
    );
}
