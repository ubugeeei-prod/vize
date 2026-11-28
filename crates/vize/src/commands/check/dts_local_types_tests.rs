use std::path::Path;

use vize_l0::{FxHashSet, String};

use super::dts::global_component_declarations_content;
use super::dts_local_types::{
    LocalType, collect_local_types, qualify_local_references, reachable_local_types,
};

/// The shape `nuxi prepare` writes to `.nuxt/types/components.d.ts` in Nuxt 4.
const NUXT_COMPONENTS: &str = r#"import type { DefineComponent, SlotsType, VNodeProps } from 'vue'

type IslandSerialized<T> = T extends string | number
  ? T
  : { [K in keyof T]: IslandSerialized<T[K]> }
type IslandProps<T> = T extends new (...args: any) => { $props: infer P }
  ? new () => { $props: { [K in keyof P]: K extends keyof VNodeProps ? P[K] : P[K] & IslandSerialized<P[K]> } }
  : unknown
type IslandComponent<T> = DefineComponent<{}, {refresh: () => Promise<void>}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, SlotsType<{ fallback: { error: unknown } }>> & T & IslandProps<T>

type HydrationStrategies = {
  // Hydrates once the element is visible.
  hydrateOnVisible?: IntersectionObserverInit | true
  hydrateNever?: true
}
type LazyComponent<T> = DefineComponent<HydrationStrategies, {}, {}, {}, {}, {}, {}, { hydrated: () => void }> & T

interface _GlobalComponents {
  BaseButton: typeof import("../../app/components/base/Button.vue")['default']
  ServerProps: IslandComponent<typeof import("../../app/components/ServerProps.server.vue")['default']>
  NuxtLink: typeof import("../../node_modules/nuxt/dist/app/components/nuxt-link")['default']
  LazyBaseButton: LazyComponent<typeof import("../../app/components/base/Button.vue")['default']>
}

declare module 'vue' {
  export interface GlobalComponents extends _GlobalComponents { }
}

declare module '#app' {
  interface NuxtAppLiterals {
    islandName: "ServerProps"
  }
}

export {}
"#;

fn local(name: &str, declaration: &str) -> LocalType {
    LocalType {
        name: name.into(),
        declaration: declaration.into(),
    }
}

#[test]
fn collects_top_level_types_but_not_augmentations() {
    let locals = collect_local_types(
        "import type { Ref } from 'vue'\n\
         type A<T> = Ref<T>;\n\
         export interface B { a: A<string> }\n\
         declare type C = B\n\
         declare module 'vue' { interface GlobalComponents { X: A<number> } }\n\
         declare global { type D = string }\n",
    );

    assert_eq!(
        locals,
        vec![
            local("A", "type A<T> = Ref<T>"),
            local("B", "interface B { a: A<string> }"),
            local("C", "type C = B"),
        ]
    );
}

#[test]
fn qualifies_type_references_only() {
    let names = ["LazyComponent", "Hydration"].into_iter().collect();
    let mut used = FxHashSet::default();

    let qualified = qualify_local_references(
        "LazyComponent<typeof import('../LazyComponent.vue')['default']> & { LazyComponent: 1; Hydration?: 2 } & X.Hydration & (T extends Hydration ? 1 : 2)",
        &names,
        "NS",
        &mut used,
    );

    assert_eq!(
        qualified.as_str(),
        "NS.LazyComponent<typeof import('../LazyComponent.vue')['default']> & { LazyComponent: 1; Hydration?: 2 } & X.Hydration & (T extends NS.Hydration ? 1 : 2)"
    );
    let mut used = used.into_iter().collect::<Vec<String>>();
    used.sort();
    assert_eq!(
        used,
        vec![String::from("Hydration"), String::from("LazyComponent")]
    );
}

#[test]
fn carries_what_a_used_type_names_in_turn_in_source_order() {
    let locals = collect_local_types(NUXT_COMPONENTS);
    let used = [String::from("IslandComponent")].into_iter().collect();

    let names = reachable_local_types(&locals, used)
        .into_iter()
        .map(|local| local.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec!["IslandSerialized", "IslandProps", "IslandComponent"]
    );
}

#[test]
fn nuxt_component_members_keep_their_file_local_types() {
    let declarations = global_component_declarations_content(
        NUXT_COMPONENTS,
        Path::new("/app/.nuxt/types"),
        "__NS",
    );

    assert_eq!(
        declarations.members,
        vec![
            (
                String::from("BaseButton"),
                String::from("typeof import(\"/app/app/components/base/Button.vue\")['default']"),
            ),
            (
                String::from("ServerProps"),
                String::from(
                    "__NS.IslandComponent<typeof import(\"/app/app/components/ServerProps.server.vue\")['default']>"
                ),
            ),
            (
                String::from("NuxtLink"),
                String::from(
                    "typeof import(\"/app/node_modules/nuxt/dist/app/components/nuxt-link\")['default']"
                ),
            ),
            (
                String::from("LazyBaseButton"),
                String::from(
                    "__NS.LazyComponent<typeof import(\"/app/app/components/base/Button.vue\")['default']>"
                ),
            ),
        ]
    );
    assert_eq!(
        declarations.local_types.as_deref(),
        Some(
            "declare namespace __NS {\n\
type IslandSerialized<T> = T extends string | number\n  ? T\n  : { [K in keyof T]: IslandSerialized<T[K]> }\n\
type IslandProps<T> = T extends new (...args: any) => { $props: infer P }\n  ? new () => { $props: { [K in keyof P]: K extends keyof import('vue').VNodeProps ? P[K] : P[K] & IslandSerialized<P[K]> } }\n  : unknown\n\
type IslandComponent<T> = import('vue').DefineComponent<{}, {refresh: () => Promise<void>}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, import('vue').SlotsType<{ fallback: { error: unknown } }>> & T & IslandProps<T>\n\
type HydrationStrategies = {\n  // Hydrates once the element is visible.\n  hydrateOnVisible?: IntersectionObserverInit | true\n  hydrateNever?: true\n}\n\
type LazyComponent<T> = import('vue').DefineComponent<HydrationStrategies, {}, {}, {}, {}, {}, {}, { hydrated: () => void }> & T\n\
}"
        )
    );
}

#[test]
fn members_without_local_types_carry_no_namespace() {
    let declarations = global_component_declarations_content(
        "type Unused = string\n\
         declare module 'vue' {\n  export interface GlobalComponents {\n    Card: typeof import('./Card.vue')['default']\n  }\n}\nexport {}\n",
        Path::new("/app/.nuxt"),
        "__NS",
    );

    assert_eq!(
        declarations.members,
        vec![(
            String::from("Card"),
            String::from("typeof import('/app/.nuxt/Card.vue')['default']")
        )]
    );
    assert_eq!(declarations.local_types, None);
}

#[test]
fn qualified_references_respect_comments_unicode_conditionals_and_binding_scopes() {
    let names = ["État", "Local", "T", "Dep"].into_iter().collect();
    let mut used = FxHashSet::default();
    let source = "{ Local: number; a: true extends true ? Local : never; b: État; c: <T>(value: T) => T; d: { // user's prop\n value: Dep }; e: `literal ${Local}` }";
    let result = qualify_local_references(source, &names, "NS", &mut used);
    assert_eq!(
        result.as_str(),
        "{ Local: number; a: true extends true ? NS.Local : never; b: NS.État; c: <T>(value: T) => T; d: { // user's prop\n value: NS.Dep }; e: `literal ${NS.Local}` }"
    );
    assert_eq!(used.len(), 3);
    assert!(!used.contains("T"));
}

#[test]
fn merged_interfaces_keep_dependencies_from_every_declaration() {
    let locals = collect_local_types(
        "type A = string; type B = number; interface Shared { a: A }; interface Shared { b: B }",
    );
    let used = [String::from("Shared")].into_iter().collect();
    let reached = reachable_local_types(&locals, used);
    assert_eq!(
        reached
            .iter()
            .map(|local| local.name.as_str())
            .collect::<Vec<_>>(),
        vec!["A", "B", "Shared", "Shared"]
    );
}

#[test]
fn copied_local_types_preserve_import_keys_and_generic_bindings() {
    let result = global_component_declarations_content(
        "import type { Ref } from 'vue'; type T = string; type Wrapper = { Ref: Ref<T>; generic: <T>() => T }; declare module 'vue' { interface GlobalComponents { Card: Wrapper } } export {}",
        Path::new("/app/.nuxt"),
        "NS",
    );
    let local = result.local_types.unwrap();
    assert!(local.contains("type Wrapper = { Ref: import('vue').Ref<T>; generic: <T>() => T }"));
    assert!(local.contains("type T = string"));
}
