//! A Vue generic component's compatibility context must not widen extracted
//! slots to unknown-argument callbacks in test utilities (#12006).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn generic_component_with_emits_accepts_typed_mount_slot() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "generic-component-mount-slots-12006",
        &[
            (
                "List.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/generic-component-mount-slots-12006/List.vue.txt"
                ),
            ),
            (
                "test-utils.d.ts",
                r#"declare module '@vue/test-utils' {
  import type { VNode } from 'vue';

  type ComponentProps<T> = T extends new (...args: any) => { $props: infer P }
    ? NonNullable<P>
    : T extends (props: infer P, ...args: any) => any ? P : {};
  type ComponentSlots<T> = T extends new (...args: any) => { $slots: infer S }
    ? NonNullable<S>
    : T extends (props: any, ctx: { slots: infer S; attrs: any; emit: any }, ...args: any) => any
      ? NonNullable<S>
      : {};
  type ShimSlotReturnType<T> = T extends (...args: infer P) => any
    ? (...args: P) => any
    : never;
  type WithArray<T> = T | T[];

  export function mount<T>(component: T, options?: {
    props?: ComponentProps<T>;
    slots?: {
      [K in keyof ComponentSlots<T>]?: WithArray<
        ShimSlotReturnType<ComponentSlots<T>[K]> | string | VNode | (new () => any) | { template: string }
      >;
    };
  }): unknown;
}
"#,
            ),
            (
                "use.ts",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/generic-component-mount-slots-12006/use.ts.txt"
                ),
            ),
            (
                "package.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/generic-component-mount-slots-12006/package.json.txt"
                ),
            ),
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/generic-component-mount-slots-12006/tsconfig.json.txt"
                ),
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(
        actual,
        Some(Vec::new()),
        "the extracted slot callback must keep the authored scope parameter",
    );
}
