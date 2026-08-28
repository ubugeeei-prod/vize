//! The type-only `defineProps` family resolves through the same prop model as
//! the runtime object form (#4145): a plain type argument, `withDefaults`, a
//! props destructure, and a generic SFC.

use super::super::super::{
    create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics,
};
use super::TYPES;
use vize_s0::{String, cstr};

const TYPE_ONLY: &str = r#"<script setup lang="ts">
import type { Pagination } from './types'

defineProps<{
  modelValue?: string
  isActive?: boolean
  pagination: Pagination
}>()
</script>

<template>
  <div>{{ $props.isActive }}</div>
  <div>{{ $props.pagination.page }}</div>
  <div>{{ $props.modelValue }}</div>
</template>
"#;

const WITH_DEFAULTS: &str = r#"<script setup lang="ts">
import type { Pagination } from './types'

const props = withDefaults(
  defineProps<{
    label?: string
    isActive?: boolean
    pagination: Pagination
  }>(),
  { label: 'ready' },
)
</script>

<template>
  <div>{{ $props.label.toUpperCase() }}</div>
  <div>{{ $props.isActive }}</div>
  <div>{{ $props.pagination.page }}</div>
  <div>{{ props.label.toUpperCase() }}</div>
  <div>{{ label.toUpperCase() }}</div>
</template>
"#;

const DESTRUCTURED: &str = r#"<script setup lang="ts">
import type { Pagination } from './types'

const { label = 'ready', pagination } = defineProps<{
  label?: string
  pagination: Pagination
}>()
</script>

<template>
  <div>{{ label.toUpperCase() }}</div>
  <div>{{ pagination.page }}</div>
  <div>{{ $props.pagination.page }}</div>
  <div>{{ $props.label }}</div>
</template>
"#;

const GENERIC: &str = r#"<script setup lang="ts" generic="T extends { id: string }">
defineProps<{ items: T[]; label?: string }>()
</script>

<template>
  <div>{{ $props.items.length }}</div>
  <div v-for="item in $props.items" :key="item.id">{{ item.id }}</div>
  <div>{{ $props.label }}</div>
</template>
"#;

/// A generic SFC whose prop type is a *deferred* conditional over the type
/// parameter, defaulted through `withDefaults`, and read back through an
/// authored `as string` cast — the nuxt-ui `Accordion`/`Breadcrumb` shape.
///
/// This is the negative control for the resolved model's own helper choice.
/// Routing this through `__DefineProps` applies
/// `{ [K in __VizeBooleanKey<T>]-?: boolean }`, and `__VizeBooleanKey` cannot
/// decide `[ItemKeys<T> | undefined] extends [boolean | undefined]` while `T` is
/// open, so TypeScript also considers the branch where `labelKey` *is* a boolean
/// key and resolves it to `ItemKeys<T> & boolean`. The authored cast then fails
/// with `TS2352` on valid library code. `vue-tsc` reports nothing here.
const GENERIC_DEFERRED_KEY_DEFAULTS: &str = r#"<script lang="ts">
type IsPlainObject<T> = T extends object ? (T extends Function ? false : true) : false
type DotPathKeys<T> = IsPlainObject<T> extends true
  ? {
      [K in keyof T & string]: IsPlainObject<NonNullable<T[K]>> extends true
        ? K | `${K}.${DotPathKeys<NonNullable<T[K]>>}`
        : K
    }[keyof T & string]
  : never
export type ItemKeys<T> = (keyof Extract<T, object> & string) | DotPathKeys<Extract<T, object>>

export function get(object: Record<string, any>, path: string): string {
  return String(object[path])
}
</script>

<script setup lang="ts" generic="T extends Record<string, any>">
const props = withDefaults(
  defineProps<{
    items?: T[]
    labelKey?: ItemKeys<T>
  }>(),
  { labelKey: 'label' as never },
)
</script>

<template>
  <div v-for="(item, i) in props.items" :key="i">
    {{ get(item, props.labelKey as string) }}
    {{ get(item, $props.labelKey as string) }}
  </div>
</template>
"#;

/// A props destructure default is a *local* default: Vue leaves the prop itself
/// optional, so `vue-tsc` still reports `$props.label` as possibly undefined.
/// Resolving defaults for the template must not swallow that.
const INVALID: &str = r#"<script setup lang="ts">
import type { Pagination } from './types'

const { label = 'ready' } = defineProps<{
  label?: string
  isActive?: boolean
  pagination: Pagination
}>()
void label
</script>

<template>
  <div>{{ $props.nope }}</div>
  <div>{{ $props['is-active'] }}</div>
  <div>{{ $props.pagination.nope }}</div>
  <div>{{ $props.label.toUpperCase() }}</div>
</template>
"#;

const REPAIRED: &str = r#"<script setup lang="ts">
import type { Pagination } from './types'

const { label = 'ready' } = defineProps<{
  label?: string
  isActive?: boolean
  pagination: Pagination
}>()
void label
</script>

<template>
  <div>{{ $props.isActive }}</div>
  <div>{{ $props.pagination.page }}</div>
  <div>{{ $props.label?.toUpperCase() }}</div>
  <div>{{ label.toUpperCase() }}</div>
</template>
"#;

/// `vue-tsc` produces no diagnostic for the four valid components or the
/// repaired variant, and exactly the four below for the invalid variant:
///
/// ```text
/// src/TypeOnlyInvalid.vue(13,18): error TS2339: Property 'nope' does not exist on type 'DefineProps<LooseRequired<__VLS_Props>, "isActive">'.
/// src/TypeOnlyInvalid.vue(14,18): error TS2551: Property 'is-active' does not exist on type 'DefineProps<LooseRequired<__VLS_Props>, "isActive">'. Did you mean 'isActive'?
/// src/TypeOnlyInvalid.vue(15,29): error TS2339: Property 'nope' does not exist on type 'Pagination'.
/// src/TypeOnlyInvalid.vue(16,11): error TS18048: '__VLS_ctx.$props.label' is possibly 'undefined'.
/// ```
#[test]
fn type_only_props_and_defaults_reach_template_dollar_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "template-instance-props-type-only",
        &[
            ("src/types.ts", TYPES),
            ("src/TypeOnlyProps.vue", TYPE_ONLY),
            ("src/WithDefaultsProps.vue", WITH_DEFAULTS),
            ("src/DestructuredProps.vue", DESTRUCTURED),
            ("src/GenericProps.vue", GENERIC),
            ("src/GenericDeferredKey.vue", GENERIC_DEFERRED_KEY_DEFAULTS),
            ("src/TypeOnlyInvalid.vue", INVALID),
            ("src/TypeOnlyRepaired.vue", REPAIRED),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    let props_type = r#"Readonly<__DefineProps<Props, "isActive">>"#;
    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/TypeOnlyInvalid.vue"),
                Some(2339),
                cstr!("13:18:error Property 'nope' does not exist on type '{props_type}'."),
            ),
            (
                String::from("src/TypeOnlyInvalid.vue"),
                Some(2339),
                cstr!("15:29:error Property 'nope' does not exist on type 'Pagination'."),
            ),
            (
                String::from("src/TypeOnlyInvalid.vue"),
                Some(2551),
                cstr!(
                    "14:18:error Property 'is-active' does not exist on type '{props_type}'. Did you mean 'isActive'?"
                ),
            ),
            (
                String::from("src/TypeOnlyInvalid.vue"),
                Some(18048),
                cstr!("16:11:error '$props.label' is possibly 'undefined'."),
            ),
        ],
        "type-only, withDefaults, destructured and generic props share one model"
    );
}
