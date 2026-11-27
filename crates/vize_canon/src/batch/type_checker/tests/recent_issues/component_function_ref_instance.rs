use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

/// The parameter of a function `ref` on a component is the instance a
/// template ref of the same component holds, as `useTemplateRef` types it:
/// the exposed surface of a `<script setup>` child and the public instance of
/// an Options API child, or `null` on unmount. vue-tsc types it as
/// `Element | ComponentPublicInstance | null` and reports every member.
#[test]
fn function_refs_on_components_receive_the_component_instance() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "component-function-ref-instance",
        &[
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-function-ref-instance/tsconfig.json.txt"
                ),
            ),
            (
                "src/Exposed.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-function-ref-instance/Exposed.vue.txt"
                ),
            ),
            (
                "src/Options.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-function-ref-instance/Options.vue.txt"
                ),
            ),
            (
                "src/App.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-function-ref-instance/App.vue.txt"
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
                "9:44:error Type '$ComponentInstance | null' is not assignable to type 'number'.\nType 'null' is not assignable to type 'number'.",
            ),
            (
                "src/App.vue",
                Some(2339),
                "11:30:error Property 'missing' does not exist on type '{ $: ComponentInternalInstance; $data: {}; $props: Partial<{}> & Omit<{} & VNodeProps & AllowedComponentProps & ComponentCustomProps, never>; ... 11 more ...; $watch<T extends string | ((...args: any) => any)>(source: T, cb: ..., options?: WatchOptions<...> | undefined): WatchStopHandle; } & ... 8 more ... & Pick<...>'.",
            ),
            (
                "src/App.vue",
                Some(2339),
                "8:40:error Property 'blur' does not exist on type '$ComponentInstance'.",
            ),
        ]
    );
}
