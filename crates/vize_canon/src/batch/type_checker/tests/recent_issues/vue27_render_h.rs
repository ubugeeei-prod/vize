use std::path::Path;

use super::super::{
    BatchTypeChecker, create_project_case_without_node_modules, relative_path,
    resolve_test_tsgo_binary,
};
use crate::batch::TypeChecker;
use vize_s0::{String, cstr};

const VUE_27_TYPES: &str = r#"export interface VNode {}
export interface VNodeData {}
export interface ComponentOptions<V extends Vue = Vue> {
  render?: (h: CreateElement) => VNode | null | void;
}
export interface Vue {
  readonly $data: Record<string, any>;
  readonly $props: Record<string, any>;
  readonly $el: Element;
  readonly $options: Record<string, any>;
  readonly $parent: Vue;
  readonly $root: Vue;
  readonly $children: Vue[];
  readonly $refs: Record<string, any>;
  readonly $slots: Record<string, VNode[] | undefined>;
  readonly $scopedSlots: Record<string, unknown>;
  readonly $isServer: boolean;
  readonly $attrs: Record<string, string>;
  readonly $listeners: Record<string, Function | Function[]>;
  readonly $vnode: VNode;
  readonly $ssrContext: any;
  $mount(...args: any[]): this;
  $forceUpdate(): void;
  $destroy(): void;
  $set(...args: any[]): any;
  $delete(...args: any[]): any;
  $watch(...args: any[]): any;
  $on(...args: any[]): this;
  $once(...args: any[]): this;
  $off(...args: any[]): this;
  $emit(...args: any[]): this;
  $nextTick(...args: any[]): Promise<void>;
  $createElement: CreateElement;
}
export interface VueConstructor<V extends Vue = Vue> {
  new (...args: any[]): V;
  extend(options?: ComponentOptions<V>): VueConstructor<V>;
  nextTick(...args: any[]): Promise<void>;
  set(...args: any[]): any;
  delete(...args: any[]): any;
  directive(...args: any[]): any;
  filter(...args: any[]): any;
  component(...args: any[]): any;
  use(...args: any[]): this;
  mixin(...args: any[]): this;
  compile(...args: any[]): any;
  observable<T>(obj: T): T;
  util: { warn(...args: any[]): void };
  config: Record<string, any>;
  version: string;
}
declare const Vue: VueConstructor;
export default Vue;
export { Vue };
export type FunctionalComponentOptions<Props = Record<string, any>> = (props: Props) => VNode;
export type DefineComponent<Props = Record<string, any>> = ComponentOptions<Vue> & {
  readonly __props?: Props;
};
export type Component = typeof Vue | FunctionalComponentOptions<any> | ComponentOptions<any> | DefineComponent<any>;
export type AsyncComponent = () => Promise<Component>;
export interface CreateElement {
  (tag?: string | Component | AsyncComponent | (() => Component), children?: any): VNode;
  (tag?: string | Component | AsyncComponent | (() => Component), data?: VNodeData, children?: any): VNode;
}
"#;

#[test]
fn vue27_render_function_accepts_imported_sfc_in_h() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case_without_node_modules(
        "vue27-render-h-imported-sfc",
        &[
            (
                "src/ReproComponent.vue",
                r#"<script setup lang="ts">
defineProps<{ label?: string }>()
</script>

<template>
  <div>{{ label }}</div>
</template>
"#,
            ),
            (
                "src/main.ts",
                r#"import Vue from 'vue';
import ReproComponent from './ReproComponent.vue';

export default Vue.extend({
  render: (h) => h(ReproComponent),
});
"#,
            ),
        ],
    );
    write_vue27_stub(&project_root);

    let snapshot = vue27_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        snapshot,
        vec![],
        "Vue 2.7 render functions should accept generated SFC constructors in h()"
    );
}

#[test]
fn vue27_render_function_accepts_imported_generic_sfc_in_h() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case_without_node_modules(
        "vue27-render-h-imported-generic-sfc",
        &[
            (
                "src/GenericComponent.vue",
                r#"<script setup lang="ts" generic="T = string">
defineProps<{ value?: T }>()
</script>

<template>
  <div>{{ value }}</div>
</template>
"#,
            ),
            (
                "src/main.ts",
                r#"import Vue from 'vue';
import GenericComponent from './GenericComponent.vue';

export default Vue.extend({
  render: (h) => h(GenericComponent),
});
"#,
            ),
        ],
    );
    write_vue27_stub(&project_root);

    let snapshot = vue27_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        snapshot,
        vec![],
        "Vue 2.7 render functions should accept generated generic SFC constructors in h()"
    );
}

fn vue27_diagnostics(project_root: &Path) -> Vec<(String, Option<u32>, String)> {
    let mut checker = BatchTypeChecker::new(project_root).expect("batch type checker construction");
    checker.set_dialect(vize_carton::config::VueVersion::V2_7);
    checker.scan_project().expect("project scan");
    let result = checker.check_project().expect("project check");

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}:{} {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    match diagnostic.severity {
                        1 => "error",
                        2 => "warning",
                        3 => "info",
                        _ => "hint",
                    },
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    snapshot
}

fn write_vue27_stub(project_root: &Path) {
    let vue_dir = project_root.join("node_modules/vue/types");
    std::fs::create_dir_all(&vue_dir).expect("create Vue 2.7 stub types directory");
    std::fs::write(
        project_root.join("node_modules/vue/package.json"),
        r#"{"name":"vue","version":"2.7.16","types":"types/index.d.ts"}"#,
    )
    .expect("write Vue 2.7 package metadata");
    std::fs::write(vue_dir.join("index.d.ts"), VUE_27_TYPES).expect("write Vue 2.7 types");
}
