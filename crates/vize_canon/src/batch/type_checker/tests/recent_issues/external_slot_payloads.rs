//! Scoped-slot payloads coming from an external component (#4147).
//!
//! Every expectation here is the *complete* diagnostic list a `vue-tsc` 3.3.4 /
//! TypeScript 6.0.3 run produced over byte-identical sources, so a payload that
//! silently widens shows up as a missing entry and one that wrongly narrows
//! shows up as an extra entry. The sources are reductions of the two release
//! parity findings the issue names: Misskey's `MkDraggable` / `MkTl` slots and
//! Vuestic Admin's `VaSelect #content` loop.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_s0::String;

mod v_for_any;

/// `<script setup generic>` child whose slot payload is its type parameter.
const GENERIC_LIST: &str = r#"<script setup lang="ts" generic="T extends { id: string }">
defineSlots<{
  default(props: { item: T; index: number }): any;
  header(props: { total: number }): any;
  footer(): any;
}>();

defineProps<{ items: T[]; label: string }>();
</script>

<template>
  <div>
    <slot name="header" :total="1" />
    <div v-for="(item, i) in items" :key="item.id">
      <slot :item="item" :index="i" />
    </div>
    <slot name="footer" />
  </div>
</template>
"#;

/// Unconstrained parameter: an uninferable payload is `unknown`, not `any`.
const UNCONSTRAINED_LIST: &str = r#"<script setup lang="ts" generic="T">
defineSlots<{
  row(props: { event: T }): any;
}>();

defineProps<{ events: T[] }>();
</script>

<template>
  <div>
    <div v-for="(event, i) in events" :key="i">
      <slot name="row" :event="event" />
    </div>
  </div>
</template>
"#;

/// Declaration-only and package-style children, typed only through `$slots`.
///
/// The declaration-only child lives in a `.ts` module rather than a `.d.ts`
/// one: a `.vue` importing a sibling `.d.ts` currently resolves to nothing in
/// the materialized virtual project, which degrades everything it exports to
/// `any` — a separate defect from this one, recorded in the PR.
const DECLARED_CHILDREN: &str = r#"import type { DefineComponent } from 'vue';

export declare const DeclaredList: DefineComponent<{ rows: string[] }> & {
  new (): {
    $slots: {
      default(props: { row: string; index: number }): any;
      empty(props: { reason: string }): any;
    };
  };
};
"#;

const PACKAGE_CHILD: &str = r#"declare module 'fancy-ui' {
  export const FancyList: import('vue').DefineComponent<{ entries: number[] }> & {
    new (): {
      $slots: {
        default(props: { entry: number; position: number }): any;
      };
    };
  };
}
"#;

#[test]
fn generic_child_slot_payload_is_instantiated_from_the_authored_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "external-slot-payload-generic",
        &[
            ("src/GenericList.vue", GENERIC_LIST),
            ("src/UnconstrainedList.vue", UNCONSTRAINED_LIST),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import GenericList from './GenericList.vue';
import UnconstrainedList from './UnconstrainedList.vue';

type Block = { id: string; kind: 'text'; text: string } | { id: string; kind: 'image'; url: string };

const blocks: Block[] = [];
const untyped: any = null;

function takesNumber(value: number) {
  return value;
}
</script>

<template>
  <GenericList :items="blocks" label="ok">
    <template #default="{ item, index }">
      <span v-if="item.kind === 'text'">{{ item.text }}</span>
      <span v-else>{{ item.url }}</span>
      <span>{{ takesNumber(index) }}</span>
    </template>
    <template #header="{ total }">{{ takesNumber(total) }}</template>
    <template #footer>done</template>
  </GenericList>

  <GenericList :items="blocks" label="alias">
    <template #default="{ item: row }">{{ row.kind }}</template>
  </GenericList>

  <GenericList :items="blocks" label="outer">
    <template #default="{ item: outer }">
      <GenericList :items="blocks" label="inner">
        <template #default="{ item: inner }">{{ outer.kind }}{{ inner.kind }}</template>
      </GenericList>
    </template>
  </GenericList>

  <GenericList :items="untyped" label="degraded">
    <template #default="{ item }">{{ item.kind }}</template>
  </GenericList>

  <UnconstrainedList :events="untyped">
    <template #row="{ event }">{{ event.id }}</template>
  </UnconstrainedList>
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    let _ = std::fs::remove_dir_all(&project_root);

    // vue-tsc reports exactly these two and nothing else: the discriminated
    // union, the aliased binding, the nested payloads, the named slot and the
    // `v-for` index of the inferable usages all check clean, while the two
    // usages whose props cannot determine the parameter fall back to the
    // constraint (`{ id: string }`) and to `unknown` respectively.
    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/App.vue"),
                Some(2339),
                String::from(
                    "39:43:error Property 'kind' does not exist on type '{ id: string; }'."
                ),
            ),
            (
                String::from("src/App.vue"),
                Some(18046),
                String::from("43:35:error 'event' is of type 'unknown'."),
            ),
        ]
    );
}

#[test]
fn declaration_only_and_package_child_slot_payloads_stay_exact() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "external-slot-payload-declared",
        &[
            ("src/declared.ts", DECLARED_CHILDREN),
            ("src/fancy-ui.d.ts", PACKAGE_CHILD),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import { DeclaredList } from './declared';
import { FancyList } from 'fancy-ui';

function takesNumber(value: number) {
  return value;
}
function takesString(value: string) {
  return value;
}
</script>

<template>
  <DeclaredList :rows="['a']">
    <template #default="{ row, index }">
      <span v-if="index > 0">{{ takesString(row) }}{{ takesNumber(index) }}</span>
    </template>
    <template #empty="{ reason }">{{ takesString(reason) }}</template>
  </DeclaredList>

  <DeclaredList :rows="['a']">
    <template #default="{ row }">{{ takesNumber(row) }}</template>
  </DeclaredList>

  <FancyList :entries="[1]">
    <template #default="{ entry, position }">{{ takesNumber(entry) }}{{ takesNumber(position) }}</template>
  </FancyList>

  <FancyList :entries="[1]">
    <template #default="{ entry }">{{ takesString(entry) }}</template>
  </FancyList>
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    let _ = std::fs::remove_dir_all(&project_root);

    // The two deliberate misuses are reported and nothing else: the `v-if`
    // guard, the named `#empty` slot and the exact `row`/`index`/`entry`/
    // `position` payload members all check clean.
    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/App.vue"),
                Some(2345),
                String::from(
                    "22:49:error Argument of type 'string' is not assignable to parameter of type 'number'."
                ),
            ),
            (
                String::from("src/App.vue"),
                Some(2345),
                String::from(
                    "30:51:error Argument of type 'number' is not assignable to parameter of type 'string'."
                ),
            ),
        ]
    );
}

#[test]
fn unnamed_inner_slot_does_not_shadow_outer_scoped_slot_payload() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "external-slot-payload-unnamed-inner-slot",
        &[
            (
                "src/List.vue",
                r#"<script setup lang="ts">
defineSlots<{
  default(props: { item: { title: string }; index: number }): any;
}>();

defineProps<{ items: Array<{ title: string }> }>();
</script>

<template>
  <div v-for="(item, index) in items" :key="item.title">
    <slot :item="item" :index="index" />
  </div>
</template>
"#,
            ),
            (
                "src/Card.vue",
                r#"<script setup lang="ts">
defineSlots<{
  title(): any;
}>();
</script>

<template>
  <section>
    <slot name="title" />
  </section>
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import List from './List.vue';
import Card from './Card.vue';

const items = [{ title: 'Ready' }];

function takesString(value: string) {
  return value;
}
</script>

<template>
  <List :items="items">
    <template #default="slotProps">
      <Card>
        <template #title>{{ takesString(slotProps.item.title) }}</template>
      </Card>
    </template>
  </List>
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    let _ = std::fs::remove_dir_all(&project_root);

    assert!(
        snapshot.is_empty(),
        "a slot without authored props must not shadow outer scoped-slot payloads: {snapshot:#?}"
    );
}
