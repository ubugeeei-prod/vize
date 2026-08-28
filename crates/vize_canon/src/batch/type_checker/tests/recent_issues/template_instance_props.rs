//! The template's `$props` is the component's own prop contract (#4145).
//!
//! `$props` used to be emitted as a generic instance global,
//! `__VizeInstanceGlobal<'$props'>`, which reads
//! `ComponentPublicInstance['$props']` with that helper's *default* type
//! arguments. `P` defaults to `{}` there, so every declared prop read through
//! `$props` reported `TS2339 … does not exist on type '{}'` — 19 of them across
//! four unchanged Vuestic Admin SFCs.
//!
//! Oracle: `vue-tsc` 3.3.x with the installed `vue`, run over byte-identical
//! fixtures. Each expectation below is the complete list that run produced, with
//! the authored line and column. Where the message text diverges it is only the
//! *name* of the props object type — `vue-tsc` prints the fully expanded
//! `{ readonly pagination: Pagination; readonly isActive: boolean; }` /
//! `DefineProps<LooseRequired<__VLS_Props>, "isActive">` where vize prints its
//! own equivalent alias — and the `__VLS_ctx.` expression prefix vize does not
//! synthesize. Codes, positions and diagnostic counts are identical.

mod options_api;
mod type_only;

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use std::path::Path;
use vize_s0::{String, cstr};

pub(super) const TYPES: &str = r#"export type Pagination = { page: number; perPage: number; total: number }
export type Project = { name: string; status: 'important' | 'archived' }
"#;

/// `va-timeline-item.vue`: a constructor-only `String` prop with a `default:`,
/// plus a defaulted `Boolean`. `vue-tsc` reports nothing here.
const TIMELINE_ITEM: &str = r#"<script setup lang="ts">
defineProps({
  date: { type: String, default: '' },
  active: { type: Boolean, default: false },
})
</script>

<template>
  <div :class="{ active: $props.active }">{{ $props.date.toUpperCase() }}</div>
</template>
"#;

/// `ProjectStatusBadge.vue`: an imported `PropType<T>` cast over a string union,
/// read through `$props` as an index into a `Record` keyed by that union — the
/// shape that produced the reported `TS7053` follow-on. `vue-tsc` reports
/// nothing here.
const STATUS_BADGE: &str = r#"<script setup lang="ts">
import type { PropType } from 'vue'
import type { Project } from './types'

defineProps({
  status: { type: String as PropType<Project['status']>, required: true },
})

const colors: Record<Project['status'], string> = {
  important: 'danger',
  archived: 'gray',
}
</script>

<template>
  <span :class="colors[$props.status]">{{ $props.status }}</span>
</template>
"#;

/// `ProjectsTable.vue` / `UsersTable.vue`: a required nested object prop read
/// and mutated through `$props`, next to a defaulted `Boolean`. The local
/// `props` binding and the bare template identifier are the non-regression
/// controls. `vue-tsc` reports nothing here.
const PROJECTS_TABLE: &str = r#"<script setup lang="ts">
import type { PropType } from 'vue'
import type { Pagination } from './types'

const props = defineProps({
  pagination: { type: Object as PropType<Pagination>, required: true },
  loading: { type: Boolean, default: false },
})
</script>

<template>
  <div v-if="$props.loading">loading</div>
  <div>{{ $props.pagination.page }}/{{ $props.pagination.perPage }} of {{ $props.pagination.total }}</div>
  <button @click="$props.pagination.page = $props.pagination.page + 1">next</button>
  <span>{{ props.pagination.page }}</span>
  <span>{{ loading }}</span>
</template>
"#;

/// The negative control: `$props` must stay exactly as narrow as `vue-tsc`'s.
const INVALID_PROPS: &str = r#"<script setup lang="ts">
import type { PropType } from 'vue'
import type { Pagination } from './types'

defineProps({
  isActive: { type: Boolean, default: false },
  pagination: { type: Object as PropType<Pagination>, required: true },
})
</script>

<template>
  <div>{{ $props.nope }}</div>
  <div>{{ $props['is-active'] }}</div>
  <div>{{ $props.pagination.nope }}</div>
  <button @click="$props.pagination.page = 'x'">a</button>
  <button @click="$props.isActive = true">b</button>
</template>
"#;

/// The same component with every mistake repaired: clean in both tools.
const REPAIRED_PROPS: &str = r#"<script setup lang="ts">
import type { PropType } from 'vue'
import type { Pagination } from './types'

defineProps({
  isActive: { type: Boolean, default: false },
  pagination: { type: Object as PropType<Pagination>, required: true },
})
</script>

<template>
  <div>{{ $props.pagination.total }}</div>
  <div>{{ $props.isActive }}</div>
  <div>{{ $props.pagination.perPage }}</div>
  <button @click="$props.pagination.page = 2">a</button>
  <button @click="$props.pagination.page = $props.pagination.page + 1">b</button>
</template>
"#;

/// The public instance contract a downstream consumer sees must stay the same
/// prop model the template resolved.
const PUBLIC_INSTANCE: &str = r#"import ProjectsTable from './ProjectsTable.vue'
import TimelineItem from './TimelineItem.vue'
import type { Pagination } from './types'

type TableProps = InstanceType<typeof ProjectsTable>['$props']
type TimelineProps = InstanceType<typeof TimelineItem>['$props']

const pagination: Pagination = { page: 1, perPage: 10, total: 20 }

export const table: TableProps = { pagination, loading: true }
export const timeline: TimelineProps = { date: 'today', active: true }
export const page: number = table.pagination.page
"#;

const TSX_CONSUMER: &str = r#"import ProjectsTable from './ProjectsTable.vue'
import TimelineItem from './TimelineItem.vue'
import type { Pagination } from './types'

const pagination: Pagination = { page: 1, perPage: 10, total: 20 }

export const rendered = (
  <>
    <ProjectsTable pagination={pagination} loading />
    <TimelineItem date="today" active />
  </>
)
"#;

fn write_jsx_tsconfig(project_root: &Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
}

/// The runtime object form of `defineProps`: the exact shapes the Vuestic Admin
/// sweep reported. `vue-tsc` produces no diagnostic for the three valid
/// components, the repaired variant, or the public instance consumer, and
/// exactly the five below for the invalid variant:
///
/// ```text
/// src/InvalidProps.vue(12,18): error TS2339: Property 'nope' does not exist on type '{ readonly pagination: Pagination; readonly isActive: boolean; }'.
/// src/InvalidProps.vue(13,18): error TS2551: Property 'is-active' does not exist on type '{ readonly pagination: Pagination; readonly isActive: boolean; }'. Did you mean 'isActive'?
/// src/InvalidProps.vue(14,29): error TS2339: Property 'nope' does not exist on type 'Pagination'.
/// src/InvalidProps.vue(15,19): error TS2322: Type 'string' is not assignable to type 'number'.
/// src/InvalidProps.vue(16,26): error TS2540: Cannot assign to 'isActive' because it is a read-only property.
/// ```
#[test]
fn runtime_object_props_reach_template_dollar_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "template-instance-props-runtime",
        &[
            ("src/types.ts", TYPES),
            ("src/TimelineItem.vue", TIMELINE_ITEM),
            ("src/StatusBadge.vue", STATUS_BADGE),
            ("src/ProjectsTable.vue", PROJECTS_TABLE),
            ("src/InvalidProps.vue", INVALID_PROPS),
            ("src/RepairedProps.vue", REPAIRED_PROPS),
            ("src/PublicInstance.ts", PUBLIC_INSTANCE),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    let props_type =
        r#"Readonly<Omit<__LooseRequired<Props>, "isActive"> & { isActive: boolean; }>"#;
    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/InvalidProps.vue"),
                Some(2322),
                cstr!("15:19:error Type 'string' is not assignable to type 'number'."),
            ),
            (
                String::from("src/InvalidProps.vue"),
                Some(2339),
                cstr!("12:18:error Property 'nope' does not exist on type '{props_type}'."),
            ),
            (
                String::from("src/InvalidProps.vue"),
                Some(2339),
                cstr!("14:29:error Property 'nope' does not exist on type 'Pagination'."),
            ),
            (
                String::from("src/InvalidProps.vue"),
                Some(2540),
                cstr!(
                    "16:26:error Cannot assign to 'isActive' because it is a read-only property."
                ),
            ),
            (
                String::from("src/InvalidProps.vue"),
                Some(2551),
                cstr!(
                    "13:18:error Property 'is-active' does not exist on type '{props_type}'. Did you mean 'isActive'?"
                ),
            ),
        ],
        "runtime `defineProps` must reach `$props` without widening it"
    );
}

/// TSX call sites keep consuming the same components through the JSX input
/// contract. `vue-tsc` reports nothing for this project.
#[test]
fn tsx_consumers_stay_green_alongside_template_dollar_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "template-instance-props-tsx",
        &[
            ("src/types.ts", TYPES),
            ("src/TimelineItem.vue", TIMELINE_ITEM),
            ("src/ProjectsTable.vue", PROJECTS_TABLE),
            ("src/Consumer.tsx", TSX_CONSUMER),
        ],
    );
    write_jsx_tsconfig(&project_root);

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot,
        Vec::new(),
        "TSX consumers must stay green while `$props` carries the prop model"
    );
}
